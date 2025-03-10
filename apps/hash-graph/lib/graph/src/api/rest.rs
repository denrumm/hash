//! The Axum webserver for accessing the Graph API operations.
//!
//! Handler methods are grouped by routes that make up the REST API.

#[cfg(all(hash_graph_test_environment, feature = "test-server"))]
#[doc(hidden)]
pub mod test_server;

mod api_resource;
mod json;
mod middleware;
mod status;
mod utoipa_typedef;

mod account;
mod data_type;
mod entity;
mod entity_type;
mod property_type;

use std::{fs, io, sync::Arc};

use async_trait::async_trait;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use error_stack::{Report, ResultExt};
use graph_types::{
    ontology::{
        CustomEntityTypeMetadata, CustomOntologyMetadata, EntityTypeMetadata,
        OntologyElementMetadata, OntologyTemporalMetadata, OntologyTypeRecordId,
        OntologyTypeReference, OntologyTypeVersion,
    },
    provenance::{OwnedById, ProvenanceMetadata, RecordArchivedById, RecordCreatedById},
};
use include_dir::{include_dir, Dir};
use temporal_versioning::{
    ClosedTemporalBound, DecisionTime, LeftClosedTemporalInterval, LimitedTemporalBound,
    OpenTemporalBound, RightBoundedTemporalInterval, TemporalBound, Timestamp, TransactionTime,
};
use utoipa::{
    openapi::{
        self, schema, ArrayBuilder, KnownFormat, Object, ObjectBuilder, OneOfBuilder, Ref, RefOr,
        Schema, SchemaFormat, SchemaType,
    },
    Modify, OpenApi, ToSchema,
};

use self::{api_resource::RoutedResource, middleware::span_trace_layer};
use crate::{
    api::rest::{
        middleware::log_request_and_response,
        utoipa_typedef::{
            subgraph::{
                Edges, KnowledgeGraphOutwardEdge, KnowledgeGraphVertex, KnowledgeGraphVertices,
                OntologyOutwardEdge, OntologyTypeVertexId, OntologyVertex, OntologyVertices,
                Subgraph, Vertex, Vertices,
            },
            MaybeListOfEntityTypeMetadata, MaybeListOfOntologyElementMetadata,
        },
    },
    ontology::{domain_validator::DomainValidator, Selector},
    store::{error::VersionedUrlAlreadyExists, QueryError, Store, StorePool, TypeFetcher},
    subgraph::{
        edges::{
            EdgeResolveDepths, GraphResolveDepths, KnowledgeGraphEdgeKind, OntologyEdgeKind,
            OutgoingEdgeResolveDepth, SharedEdgeKind,
        },
        identifier::{
            DataTypeVertexId, EntityIdWithInterval, EntityTypeVertexId, EntityVertexId,
            GraphElementVertexId, PropertyTypeVertexId,
        },
        temporal_axes::{
            QueryTemporalAxes, QueryTemporalAxesUnresolved, RightBoundedTemporalIntervalUnresolved,
            SubgraphTemporalAxes,
        },
    },
};

#[async_trait]
pub trait RestApiStore: Store + TypeFetcher {
    async fn load_external_type(
        &mut self,
        domain_validator: &DomainValidator,
        reference: OntologyTypeReference<'_>,
        actor_id: RecordCreatedById,
    ) -> Result<OntologyElementMetadata, StatusCode>;
}

#[async_trait]
impl<S> RestApiStore for S
where
    S: Store + TypeFetcher + Send,
{
    async fn load_external_type(
        &mut self,
        domain_validator: &DomainValidator,
        reference: OntologyTypeReference<'_>,
        actor_id: RecordCreatedById,
    ) -> Result<OntologyElementMetadata, StatusCode> {
        if domain_validator.validate_url(reference.url().base_url.as_str()) {
            tracing::error!(id=%reference.url(), "Ontology type is not external");
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }

        self
            .insert_external_ontology_type(
                reference,
                actor_id,
            )
            .await
            .map_err(|report| {
                tracing::error!(error=?report, id=%reference.url(), "Could not insert external type");
                if report.contains::<VersionedUrlAlreadyExists>() {
                    StatusCode::CONFLICT
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            })
    }
}

static STATIC_SCHEMAS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/api/rest/json_schemas");

fn api_resources<P: StorePool + Send + 'static>() -> Vec<Router>
where
    for<'pool> P::Store<'pool>: RestApiStore,
{
    vec![
        account::AccountResource::routes::<P>(),
        data_type::DataTypeResource::routes::<P>(),
        property_type::PropertyTypeResource::routes::<P>(),
        entity_type::EntityTypeResource::routes::<P>(),
        entity::EntityResource::routes::<P>(),
    ]
}

fn api_documentation() -> Vec<openapi::OpenApi> {
    vec![
        account::AccountResource::documentation(),
        data_type::DataTypeResource::documentation(),
        property_type::PropertyTypeResource::documentation(),
        entity_type::EntityTypeResource::documentation(),
        entity::EntityResource::documentation(),
    ]
}

fn report_to_status_code<C>(report: &Report<C>) -> StatusCode {
    let mut status_code = StatusCode::INTERNAL_SERVER_ERROR;

    if let Some(error) = report.downcast_ref::<QueryError>() {
        tracing::error!(%error, "Unable to query from data store");
        status_code = StatusCode::UNPROCESSABLE_ENTITY;
    }
    status_code
}

pub struct RestRouterDependencies<P: StorePool + Send + 'static> {
    pub store: Arc<P>,
    pub domain_regex: DomainValidator,
}

/// A [`Router`] that only serves the `OpenAPI` specification (JSON, and necessary subschemas) for
/// the REST API.
pub fn openapi_only_router() -> Router {
    let open_api_doc = OpenApiDocumentation::openapi();

    Router::new().nest(
        "/api-doc",
        Router::new()
            .route("/openapi.json", get(|| async { Json(open_api_doc) }))
            .route("/models/*path", get(serve_static_schema)),
    )
}

/// A [`Router`] that serves all of the REST API routes, and the `OpenAPI` specification.
pub fn rest_api_router<P: StorePool + Send + 'static>(
    dependencies: RestRouterDependencies<P>,
) -> Router
where
    for<'pool> P::Store<'pool>: RestApiStore,
{
    // All api resources are merged together into a super-router.
    let merged_routes = api_resources::<P>()
        .into_iter()
        .fold(Router::new(), Router::merge);

    // super-router can then be used as any other router.
    // Make sure extensions are added at the end so they are made available to merged routers.
    // The `/api-doc` endpoints are nested as we don't want any layers or handlers for the api-doc
    merged_routes
        .layer(Extension(dependencies.store))
        .layer(Extension(dependencies.domain_regex))
        .layer(axum::middleware::from_fn(log_request_and_response))
        .layer(span_trace_layer())
        .merge(openapi_only_router())
}

async fn serve_static_schema(Path(path): Path<String>) -> Result<Response, StatusCode> {
    let path = path.trim_start_matches('/');

    STATIC_SCHEMAS
        .get_file(path)
        .map_or(Err(StatusCode::NOT_FOUND), |file| {
            Ok((
                [(
                    axum::http::header::CONTENT_TYPE,
                    axum::http::HeaderValue::from_static("application/json"),
                )],
                file.contents(),
            )
                .into_response())
        })
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Graph", description = "HASH Graph API")
    ),
    modifiers(
        &MergeAddon,
        &ExternalRefAddon,
        &OperationGraphTagAddon,
        &FilterSchemaAddon,
        &TimeSchemaAddon,
    ),
    components(
        schemas(
            OwnedById,
            RecordCreatedById,
            RecordArchivedById,
            ProvenanceMetadata,
            OntologyTypeRecordId,
            OntologyElementMetadata,
            OntologyTemporalMetadata,
            CustomOntologyMetadata,
            EntityTypeMetadata,
            CustomEntityTypeMetadata,
            MaybeListOfOntologyElementMetadata,
            MaybeListOfEntityTypeMetadata,
            EntityVertexId,
            EntityIdWithInterval,
            DataTypeVertexId,
            PropertyTypeVertexId,
            EntityTypeVertexId,
            OntologyTypeVertexId,
            OntologyTypeVersion,
            Selector,

            GraphElementVertexId,
            OntologyVertex,
            KnowledgeGraphVertex,
            Vertex,
            KnowledgeGraphVertices,
            OntologyVertices,
            Vertices,
            SharedEdgeKind,
            KnowledgeGraphEdgeKind,
            OntologyEdgeKind,
            OntologyOutwardEdge,
            KnowledgeGraphOutwardEdge,
            Edges,
            GraphResolveDepths,
            EdgeResolveDepths,
            OutgoingEdgeResolveDepth,
            Subgraph,
            SubgraphTemporalAxes,

            DecisionTime,
            TransactionTime,
            QueryTemporalAxes,
            QueryTemporalAxesUnresolved,
        )
    ),
)]
pub struct OpenApiDocumentation;

impl OpenApiDocumentation {
    /// Writes the `OpenAPI` specification to the given path.
    ///
    /// The path must be a directory, and the `OpenAPI` specification will be written to
    /// `openapi.json` in that directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if the path is not a directory, or if the files cannot be
    /// written.
    pub fn write_openapi(path: impl AsRef<std::path::Path>) -> Result<(), Report<io::Error>> {
        let openapi = Self::openapi();
        let path = path.as_ref();
        fs::create_dir_all(path).attach_printable_lazy(|| path.display().to_string())?;

        let openapi_json_path = path.join("openapi.json");
        serde_json::to_writer_pretty(
            io::BufWriter::new(
                fs::File::create(&openapi_json_path)
                    .attach_printable("could not write openapi.json")
                    .attach_printable_lazy(|| openapi_json_path.display().to_string())?,
            ),
            &openapi,
        )
        .map_err(io::Error::from)?;

        let model_def_path = std::path::Path::new(&env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("api")
            .join("rest")
            .join("json_schemas");

        let model_path_dir = path.join("models");
        fs::create_dir_all(&model_path_dir)
            .attach_printable("could not create directory")
            .attach_printable_lazy(|| model_path_dir.display().to_string())?;

        for file in STATIC_SCHEMAS.files() {
            let model_path_source = model_def_path.join(file.path());
            let model_path_target = model_path_dir.join(file.path());
            fs::copy(&model_path_source, &model_path_target)
                .attach_printable("could not copy file")
                .attach_printable_lazy(|| model_path_source.display().to_string())
                .attach_printable_lazy(|| model_path_target.display().to_string())?;
        }

        Ok(())
    }
}

/// Addon to merge multiple [`OpenApi`] documents together.
///
/// [`OpenApi`]: utoipa::openapi::OpenApi
struct MergeAddon;

impl Modify for MergeAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let api_documentation = api_documentation();

        let api_components = api_documentation
            .iter()
            .cloned()
            .filter_map(|api_docs| {
                api_docs
                    .components
                    .map(|components| components.schemas.into_iter())
            })
            .flatten();

        let mut components = openapi.components.take().unwrap_or_default();
        components.schemas.extend(api_components);
        openapi.components = Some(components);

        let mut tags = openapi.tags.take().unwrap_or_default();
        tags.extend(
            api_documentation
                .iter()
                .cloned()
                .filter_map(|api_docs| api_docs.tags)
                .flatten(),
        );
        openapi.tags = Some(tags);

        openapi.paths.paths.extend(
            api_documentation
                .iter()
                .cloned()
                .flat_map(|api_docs| api_docs.paths.paths.into_iter()),
        );
    }
}

/// Addon to allow external references in schemas.
///
/// Any component that starts with `VAR_` will transform into a relative URL in the schema and
/// receive a `.json` ending.
///
/// Any component that starts with `SHARED_` will transform into a relative URL into the
/// `./models/shared.json` file.
///
/// For example the `VAR_Entity` component will be transformed into `./models/Entity.json`
struct ExternalRefAddon;

impl Modify for ExternalRefAddon {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        for path_item in openapi.paths.paths.values_mut() {
            for operation in path_item.operations.values_mut() {
                if let Some(request_body) = &mut operation.request_body {
                    modify_component(request_body.content.values_mut());
                }

                for response in &mut operation.responses.responses.values_mut() {
                    match response {
                        RefOr::Ref(reference) => modify_reference(reference),
                        RefOr::T(response) => modify_component(response.content.values_mut()),
                    }
                }
            }
        }

        if let Some(components) = &mut openapi.components {
            for component in &mut components.schemas.values_mut() {
                modify_schema_references(component);
            }
        }
    }
}

fn modify_component<'a>(content_iter: impl IntoIterator<Item = &'a mut openapi::Content>) {
    for content in content_iter {
        modify_schema_references(&mut content.schema);
    }
}

fn modify_schema_references(schema_component: &mut RefOr<openapi::Schema>) {
    match schema_component {
        RefOr::Ref(reference) => modify_reference(reference),
        RefOr::T(schema) => match schema {
            openapi::Schema::Object(object) => object
                .properties
                .values_mut()
                .for_each(modify_schema_references),
            openapi::Schema::Array(array) => modify_schema_references(array.items.as_mut()),
            openapi::Schema::OneOf(one_of) => {
                one_of.items.iter_mut().for_each(modify_schema_references);
            }
            openapi::Schema::AllOf(all_of) => {
                all_of.items.iter_mut().for_each(modify_schema_references);
            }
            _ => (),
        },
    }
}

fn modify_reference(reference: &mut openapi::Ref) {
    static REF_PREFIX_MODELS: &str = "#/components/schemas/VAR_";
    static REF_PREFIX_SHARED: &str = "#/components/schemas/SHARED_";

    if reference.ref_location.starts_with(REF_PREFIX_MODELS) {
        reference
            .ref_location
            .replace_range(0..REF_PREFIX_MODELS.len(), "./models/");
        reference.ref_location.make_ascii_lowercase();
        reference.ref_location.push_str(".json");
    };

    if reference.ref_location.starts_with(REF_PREFIX_SHARED) {
        reference.ref_location.replace_range(
            0..REF_PREFIX_SHARED.len(),
            "./models/shared.json#/definitions/",
        );
    };
}

/// Append a `Graph` tag wherever a tag appears in individual routes.
///
/// When generating API clients the tags are used for grouping routes. Having the `Graph` tag on all
/// routes makes it so that every operation appear under the same `Graph` API interface.
///
/// As generators are not all created the same way, we're putting the `Graph` tag in the beginning
/// for it to take precedence. Other tags in the system are used for logical grouping of the
/// routes, which is why we don't want to entirely replace them.
struct OperationGraphTagAddon;

impl Modify for OperationGraphTagAddon {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        let tag = "Graph";

        for path_item in openapi.paths.paths.values_mut() {
            for operation in path_item.operations.values_mut() {
                if let Some(tags) = &mut operation.tags {
                    tags.insert(0, tag.to_owned());
                }
            }
        }
    }
}

struct FilterSchemaAddon;

impl Modify for FilterSchemaAddon {
    #[expect(clippy::too_many_lines)]
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        // This is a bit of hack, but basically, it adds a schema that is equivalent to "any value"
        // `SchemaType::Value` indicates any generic JSON value.
        struct Any;

        impl ToSchema<'_> for Any {
            fn schema() -> (&'static str, RefOr<Schema>) {
                (
                    "Any",
                    Schema::Object(Object::with_type(SchemaType::Value)).into(),
                )
            }
        }

        if let Some(ref mut components) = openapi.components {
            components.schemas.insert(
                "Filter".to_owned(),
                schema::Schema::OneOf(
                    OneOfBuilder::new()
                        .item(
                            ObjectBuilder::new()
                                .title(Some("AllFilter"))
                                .property(
                                    "all",
                                    ArrayBuilder::new().items(Ref::from_schema_name("Filter")),
                                )
                                .required("all"),
                        )
                        .item(
                            ObjectBuilder::new()
                                .title(Some("AnyFilter"))
                                .property(
                                    "any",
                                    ArrayBuilder::new().items(Ref::from_schema_name("Filter")),
                                )
                                .required("any"),
                        )
                        .item(
                            ObjectBuilder::new()
                                .title(Some("NotFilter"))
                                .property("not", Ref::from_schema_name("Filter"))
                                .required("not"),
                        )
                        .item(
                            ObjectBuilder::new()
                                .title(Some("EqualFilter"))
                                .property(
                                    "equal",
                                    ArrayBuilder::new()
                                        .items(Ref::from_schema_name("FilterExpression"))
                                        .min_items(Some(2))
                                        .max_items(Some(2)),
                                )
                                .required("equal"),
                        )
                        .item(
                            ObjectBuilder::new()
                                .title(Some("NotEqualFilter"))
                                .property(
                                    "notEqual",
                                    ArrayBuilder::new()
                                        .items(Ref::from_schema_name("FilterExpression"))
                                        .min_items(Some(2))
                                        .max_items(Some(2)),
                                )
                                .required("notEqual"),
                        )
                        .item(
                            ObjectBuilder::new()
                                .title(Some("StartsWithFilter"))
                                .property(
                                    "startsWith",
                                    ArrayBuilder::new()
                                        .items(Ref::from_schema_name("FilterExpression"))
                                        .min_items(Some(2))
                                        .max_items(Some(2)),
                                )
                                .required("startsWith"),
                        )
                        .item(
                            ObjectBuilder::new()
                                .title(Some("EndsWithFilter"))
                                .property(
                                    "endsWith",
                                    ArrayBuilder::new()
                                        .items(Ref::from_schema_name("FilterExpression"))
                                        .min_items(Some(2))
                                        .max_items(Some(2)),
                                )
                                .required("endsWith"),
                        )
                        .item(
                            ObjectBuilder::new()
                                .title(Some("ContainsSegmentFilter"))
                                .property(
                                    "containsSegment",
                                    ArrayBuilder::new()
                                        .items(Ref::from_schema_name("FilterExpression"))
                                        .min_items(Some(2))
                                        .max_items(Some(2)),
                                )
                                .required("containsSegment"),
                        )
                        .build(),
                )
                .into(),
            );
            components.schemas.insert(
                "FilterExpression".to_owned(),
                schema::Schema::OneOf(
                    OneOfBuilder::new()
                        .item(
                            ObjectBuilder::new()
                                .title(Some("PathExpression"))
                                .property(
                                    "path",
                                    ArrayBuilder::new().items(
                                        OneOfBuilder::new()
                                            .item(Ref::from_schema_name("DataTypeQueryToken"))
                                            .item(Ref::from_schema_name("PropertyTypeQueryToken"))
                                            .item(Ref::from_schema_name("EntityTypeQueryToken"))
                                            .item(Ref::from_schema_name("EntityQueryToken"))
                                            .item(Ref::from_schema_name("Selector"))
                                            .item(
                                                ObjectBuilder::new()
                                                    .schema_type(SchemaType::String),
                                            )
                                            .item(
                                                ObjectBuilder::new()
                                                    .schema_type(SchemaType::Number),
                                            ),
                                    ),
                                )
                                .required("path"),
                        )
                        .item(
                            ObjectBuilder::new()
                                .title(Some("ParameterExpression"))
                                .property("parameter", Any::schema().1)
                                .required("parameter"),
                        )
                        .build(),
                )
                .into(),
            );
        }
    }
}

/// Adds time-related structs to the `OpenAPI` schema.
struct TimeSchemaAddon;

impl Modify for TimeSchemaAddon {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        if let Some(ref mut components) = openapi.components {
            components.schemas.insert(
                Timestamp::<()>::schema().0.to_owned(),
                Timestamp::<()>::schema().1,
            );
            components.schemas.insert(
                "NullableTimestamp".to_owned(),
                ObjectBuilder::new()
                    .schema_type(SchemaType::String)
                    .format(Some(SchemaFormat::KnownFormat(KnownFormat::DateTime)))
                    .nullable(true)
                    .into(),
            );
            components.schemas.insert(
                TemporalBound::<()>::schema().0.to_owned(),
                TemporalBound::<()>::schema().1,
            );
            components.schemas.insert(
                LimitedTemporalBound::<()>::schema().0.to_owned(),
                LimitedTemporalBound::<()>::schema().1,
            );
            components.schemas.insert(
                OpenTemporalBound::<()>::schema().0.to_owned(),
                OpenTemporalBound::<()>::schema().1,
            );
            components.schemas.insert(
                ClosedTemporalBound::<()>::schema().0.to_owned(),
                ClosedTemporalBound::<()>::schema().1,
            );
            components.schemas.insert(
                "LeftClosedTemporalInterval".to_owned(),
                LeftClosedTemporalInterval::<()>::schema().1,
            );
            components.schemas.insert(
                "RightBoundedTemporalInterval".to_owned(),
                RightBoundedTemporalInterval::<()>::schema().1,
            );
            components.schemas.insert(
                RightBoundedTemporalIntervalUnresolved::<()>::schema()
                    .0
                    .to_owned(),
                RightBoundedTemporalIntervalUnresolved::<()>::schema().1,
            );
        }
    }
}
