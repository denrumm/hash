use std::{fmt, str::FromStr};

use graph_types::knowledge::entity::Entity;
use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use temporal_versioning::{ClosedTemporalBound, TemporalTagged, TimeAxis};
use utoipa::ToSchema;

use crate::{
    ontology::{EntityTypeQueryPath, EntityTypeQueryPathVisitor},
    store::{
        query::{parse_query_token, JsonPath, ParameterType, PathToken, QueryPath},
        Record,
    },
    subgraph::{
        edges::{EdgeDirection, KnowledgeGraphEdgeKind, SharedEdgeKind},
        identifier::EntityVertexId,
    },
};

#[derive(Debug, PartialEq, Eq)]
pub enum EntityQueryPath<'p> {
    /// The [`EntityUuid`] of the [`EntityId`] belonging to the [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::knowledge::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["uuid"]))?;
    /// assert_eq!(path, EntityQueryPath::Uuid);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`EntityUuid`]: graph_types::knowledge::entity::EntityUuid
    /// [`EntityId`]: graph_types::knowledge::entity::EntityId
    Uuid,
    /// The [`OwnedById`] of the [`EntityId`] belonging to the [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::knowledge::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["ownedById"]))?;
    /// assert_eq!(path, EntityQueryPath::OwnedById);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`OwnedById`]: graph_types::provenance::OwnedById
    /// [`EntityId`]: graph_types::knowledge::entity::EntityId
    OwnedById,
    /// The [`EntityEditionId`] of the [`EntityRecordId`] belonging to the [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::knowledge::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["editionId"]))?;
    /// assert_eq!(path, EntityQueryPath::EditionId);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`EntityEditionId`]: graph_types::knowledge::entity::EntityEditionId
    /// [`EntityRecordId`]: graph_types::knowledge::entity::EntityRecordId
    EditionId,
    /// The decision time axis of the [`EntityTemporalMetadata`] belonging to the [`Entity`].
    ///
    /// It's not possible to query for the temporal axis directly, this has to be done via the
    /// `temporalAxes` parameter on [`StructuralQuery`]. The decision time is returned as part of
    /// [`EntityTemporalMetadata`] of the [`EntityMetadata`].
    ///
    /// [`StructuralQuery`]: crate::subgraph::query::StructuralQuery
    /// [`EntityMetadata`]: graph_types::knowledge::entity::EntityMetadata
    /// [`EntityTemporalMetadata`]: graph_types::knowledge::entity::EntityTemporalMetadata
    DecisionTime,
    /// The transaction time axis of the [`EntityTemporalMetadata`] belonging to the [`Entity`].
    ///
    /// It's not possible to query for the temporal axis directly, this has to be done via the
    /// `temporalAxes` parameter on [`StructuralQuery`]. The transaction time is returned as part
    /// of [`EntityTemporalMetadata`] of the [`EntityMetadata`].
    ///
    /// [`StructuralQuery`]: crate::subgraph::query::StructuralQuery
    /// [`EntityMetadata`]: graph_types::knowledge::entity::EntityMetadata
    /// [`EntityTemporalMetadata`]: graph_types::knowledge::entity::EntityTemporalMetadata
    TransactionTime,
    /// Whether or not the [`Entity`] is archived.
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::knowledge::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["archived"]))?;
    /// assert_eq!(path, EntityQueryPath::Archived);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    Archived,
    /// The [`RecordCreatedById`] of the [`ProvenanceMetadata`] belonging to the [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::knowledge::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["recordCreatedById"]))?;
    /// assert_eq!(path, EntityQueryPath::RecordCreatedById);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`RecordCreatedById`]: graph_types::provenance::RecordCreatedById
    /// [`ProvenanceMetadata`]: graph_types::provenance::ProvenanceMetadata
    RecordCreatedById,
    /// An edge from this [`Entity`] to it's [`EntityType`] using a [`SharedEdgeKind`].
    ///
    /// The corresponding reversed edge is [`EntityTypeQueryPath::EntityEdge`].
    ///
    /// Deserializes from `["type", ...]` where `...` is a path to a field of an [`EntityType`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::{knowledge::EntityQueryPath, ontology::EntityTypeQueryPath};
    /// # use graph::subgraph::edges::SharedEdgeKind;
    /// let path = EntityQueryPath::deserialize(json!(["type", "baseUrl"]))?;
    /// assert_eq!(path, EntityQueryPath::EntityTypeEdge {
    ///     edge_kind: SharedEdgeKind::IsOfType,
    ///     path: EntityTypeQueryPath::BaseUrl,
    ///     inheritance_depth: None,
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// It's possible to specify the inheritance search depths:
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::{knowledge::EntityQueryPath, ontology::EntityTypeQueryPath};
    /// # use graph::subgraph::edges::SharedEdgeKind;
    /// let path = EntityQueryPath::deserialize(json!(["type(inheritanceDepth = 10)", "baseUrl"]))?;
    /// assert_eq!(path, EntityQueryPath::EntityTypeEdge {
    ///     edge_kind: SharedEdgeKind::IsOfType,
    ///     path: EntityTypeQueryPath::BaseUrl,
    ///     inheritance_depth: Some(10),
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`EntityType`]: type_system::PropertyType
    EntityTypeEdge {
        edge_kind: SharedEdgeKind,
        path: EntityTypeQueryPath<'p>,
        inheritance_depth: Option<u32>,
    },
    /// An edge between two [`Entities`][`Entity`] using a [`KnowledgeGraphEdgeKind`].
    ///
    ///
    /// # Left entity
    ///
    /// Corresponds to the [`Entity`] specified by [`LinkData::left_entity_id()`].
    ///
    /// Deserializes from `["leftEntity", ...]` where `...` is the path of the left [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::knowledge::EntityQueryPath;
    /// # use graph::subgraph::edges::{EdgeDirection, KnowledgeGraphEdgeKind};
    /// let path = EntityQueryPath::deserialize(json!(["leftEntity", "uuid"]))?;
    /// assert_eq!(path, EntityQueryPath::EntityEdge {
    ///     edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
    ///     path: Box::new(EntityQueryPath::Uuid),
    ///     direction: EdgeDirection::Outgoing,
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`LinkData::left_entity_id()`]: graph_types::knowledge::link::LinkData::left_entity_id
    ///
    ///
    /// # Right entity
    ///
    /// Corresponds to the [`Entity`] specified by [`LinkData::right_entity_id()`].
    ///
    /// Deserializes from `["rightEntity", ...]` where `...` is the path of the left [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::knowledge::EntityQueryPath;
    /// # use graph::subgraph::edges::{EdgeDirection, KnowledgeGraphEdgeKind};
    /// let path = EntityQueryPath::deserialize(json!(["rightEntity", "uuid"]))?;
    /// assert_eq!(path, EntityQueryPath::EntityEdge {
    ///     edge_kind: KnowledgeGraphEdgeKind::HasRightEntity,
    ///     path: Box::new(EntityQueryPath::Uuid),
    ///     direction: EdgeDirection::Outgoing,
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`LinkData::right_entity_id()`]: graph_types::knowledge::link::LinkData::right_entity_id
    ///
    ///
    /// # Incoming links
    ///
    /// Represents an [`Entity`] linked from this [`Entity`].
    ///
    /// Deserializes from `["incomingLinks", ...]` where `...` is the path of the target
    /// [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::knowledge::EntityQueryPath;
    /// # use graph::subgraph::edges::{EdgeDirection, KnowledgeGraphEdgeKind};
    /// let path = EntityQueryPath::deserialize(json!(["incomingLinks", "uuid"]))?;
    /// assert_eq!(path, EntityQueryPath::EntityEdge {
    ///     edge_kind: KnowledgeGraphEdgeKind::HasRightEntity,
    ///     path: Box::new(EntityQueryPath::Uuid),
    ///     direction: EdgeDirection::Incoming,
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    ///
    /// # Outgoing links
    ///
    /// Represents an [`Entity`] linked from this [`Entity`].
    ///
    /// Deserializes from `["outgoingLinks", ...]` where `...` is the path of the target
    /// [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::knowledge::EntityQueryPath;
    /// # use graph::subgraph::edges::{EdgeDirection, KnowledgeGraphEdgeKind};
    /// let path = EntityQueryPath::deserialize(json!(["outgoingLinks", "uuid"]))?;
    /// assert_eq!(path, EntityQueryPath::EntityEdge {
    ///     edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
    ///     path: Box::new(EntityQueryPath::Uuid),
    ///     direction: EdgeDirection::Incoming,
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    EntityEdge {
        edge_kind: KnowledgeGraphEdgeKind,
        path: Box<Self>,
        direction: EdgeDirection,
    },
    /// Corresponds to [`EntityLinkOrder::left_to_right`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::knowledge::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["leftToRightOrder"]))?;
    /// assert_eq!(path, EntityQueryPath::LeftToRightOrder);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`EntityLinkOrder::left_to_right`]: graph_types::knowledge::link::EntityLinkOrder::left_to_right
    LeftToRightOrder,
    /// Corresponds to [`EntityLinkOrder::right_to_left`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::knowledge::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["rightToLeftOrder"]))?;
    /// assert_eq!(path, EntityQueryPath::RightToLeftOrder);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`EntityLinkOrder::right_to_left`]: graph_types::knowledge::link::EntityLinkOrder::right_to_left
    RightToLeftOrder,
    /// Corresponds to [`Entity::properties`].
    ///
    /// Deserializes from `["properties", ...]` where `...` is a path to a property URL of an
    /// [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use graph::knowledge::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!([
    ///     "properties",
    ///     "https://blockprotocol.org/@blockprotocol/types/property-type/address/",
    ///     0,
    ///     "street"
    /// ]))?;
    /// assert_eq!(
    ///     path.to_string(),
    ///     r#"properties.$."https://blockprotocol.org/@blockprotocol/types/property-type/address/"[0]."street""#
    /// );
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    Properties(Option<JsonPath<'p>>),
}

impl fmt::Display for EntityQueryPath<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uuid => fmt.write_str("uuid"),
            Self::OwnedById => fmt.write_str("ownedById"),
            Self::RecordCreatedById => fmt.write_str("recordCreatedById"),
            Self::EditionId => fmt.write_str("editionId"),
            Self::DecisionTime => fmt.write_str("decisionTime"),
            Self::TransactionTime => fmt.write_str("transactionTime"),
            Self::Archived => fmt.write_str("archived"),
            Self::Properties(Some(property)) => write!(fmt, "properties.{property}"),
            Self::Properties(None) => fmt.write_str("properties"),
            Self::EntityTypeEdge {
                edge_kind: SharedEdgeKind::IsOfType,
                path,
                inheritance_depth: Some(depth),
            } => write!(fmt, "type({depth}).{path}"),
            Self::EntityTypeEdge {
                edge_kind: SharedEdgeKind::IsOfType,
                path,
                inheritance_depth: None,
            } => write!(fmt, "type.{path}"),
            Self::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
                path,
                direction: EdgeDirection::Outgoing,
            } => write!(fmt, "leftEntity.{path}"),
            Self::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasRightEntity,
                path,
                direction: EdgeDirection::Outgoing,
            } => write!(fmt, "rightEntity.{path}"),
            Self::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
                path,
                direction: EdgeDirection::Incoming,
            } => write!(fmt, "outgoingLinks.{path}"),
            Self::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasRightEntity,
                path,
                direction: EdgeDirection::Incoming,
            } => write!(fmt, "incomingLinks.{path}"),
            Self::LeftToRightOrder => fmt.write_str("leftToRightOrder"),
            Self::RightToLeftOrder => fmt.write_str("rightToLeftOrder"),
        }
    }
}

impl QueryPath for EntityQueryPath<'_> {
    fn expected_type(&self) -> ParameterType {
        match self {
            Self::EditionId | Self::Uuid | Self::OwnedById | Self::RecordCreatedById => {
                ParameterType::Uuid
            }
            Self::DecisionTime | Self::TransactionTime => ParameterType::TimeInterval,
            Self::Properties(_) => ParameterType::Any,
            Self::LeftToRightOrder | Self::RightToLeftOrder => ParameterType::Number,
            Self::Archived => ParameterType::Boolean,
            Self::EntityTypeEdge { path, .. } => path.expected_type(),
            Self::EntityEdge { path, .. } => path.expected_type(),
        }
    }
}

/// A single token in an [`EntityQueryPath`].
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum EntityQueryToken {
    // TODO: we want to expose `EntityId` here instead
    Uuid,
    EditionId,
    Archived,
    OwnedById,
    RecordCreatedById,
    Type,
    Properties,
    IncomingLinks,
    OutgoingLinks,
    LeftEntity,
    RightEntity,
    LeftToRightOrder,
    RightToLeftOrder,
}

/// Deserializes an [`EntityQueryPath`] from a string sequence.
pub struct EntityQueryPathVisitor {
    /// The current position in the sequence when deserializing.
    position: usize,
}

impl EntityQueryPathVisitor {
    pub const EXPECTING: &'static str = "one of `uuid`, `editionId`, `archived`, `ownedById`, \
                                         `recordCreatedById`, `type`, `properties`, \
                                         `incomingLinks`, `outgoingLinks`, `leftEntity`, \
                                         `rightEntity`, `leftToRightOrder`, `rightToLeftOrder`";

    #[must_use]
    pub const fn new(position: usize) -> Self {
        Self { position }
    }
}

impl<'de> Visitor<'de> for EntityQueryPathVisitor {
    type Value = EntityQueryPath<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(Self::EXPECTING)
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let query_token: String = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(self.position, &self))?;
        let (token, mut parameters) = parse_query_token(&query_token)?;
        self.position += 1;

        let query_path = match token {
            EntityQueryToken::Uuid => EntityQueryPath::Uuid,
            EntityQueryToken::EditionId => EntityQueryPath::EditionId,
            EntityQueryToken::OwnedById => EntityQueryPath::OwnedById,
            EntityQueryToken::RecordCreatedById => EntityQueryPath::RecordCreatedById,
            EntityQueryToken::Archived => EntityQueryPath::Archived,
            EntityQueryToken::Type => EntityQueryPath::EntityTypeEdge {
                edge_kind: SharedEdgeKind::IsOfType,
                path: EntityTypeQueryPathVisitor::new(self.position).visit_seq(seq)?,
                inheritance_depth: parameters
                    .remove("inheritanceDepth")
                    .map(u32::from_str)
                    .transpose()
                    .map_err(de::Error::custom)?,
            },
            EntityQueryToken::Properties => {
                let mut path_tokens = Vec::new();
                while let Some(property) = seq.next_element::<PathToken<'de>>()? {
                    path_tokens.push(property);
                    self.position += 1;
                }

                if path_tokens.is_empty() {
                    EntityQueryPath::Properties(None)
                } else {
                    EntityQueryPath::Properties(Some(JsonPath::from_path_tokens(path_tokens)))
                }
            }
            EntityQueryToken::LeftEntity => EntityQueryPath::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
                path: Box::new(Self::new(self.position).visit_seq(seq)?),
                direction: EdgeDirection::Outgoing,
            },
            EntityQueryToken::RightEntity => EntityQueryPath::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasRightEntity,
                path: Box::new(Self::new(self.position).visit_seq(seq)?),
                direction: EdgeDirection::Outgoing,
            },
            EntityQueryToken::OutgoingLinks => EntityQueryPath::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
                path: Box::new(Self::new(self.position).visit_seq(seq)?),
                direction: EdgeDirection::Incoming,
            },
            EntityQueryToken::IncomingLinks => EntityQueryPath::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasRightEntity,
                path: Box::new(Self::new(self.position).visit_seq(seq)?),
                direction: EdgeDirection::Incoming,
            },
            EntityQueryToken::LeftToRightOrder => EntityQueryPath::LeftToRightOrder,
            EntityQueryToken::RightToLeftOrder => EntityQueryPath::RightToLeftOrder,
        };

        if !parameters.is_empty() {
            return Err(de::Error::custom(format!(
                "unknown parameters: {}",
                parameters.into_keys().collect::<Vec<_>>().join(", ")
            )));
        }

        Ok(query_path)
    }
}

impl<'de: 'p, 'p> Deserialize<'de> for EntityQueryPath<'p> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(EntityQueryPathVisitor::new(0))
    }
}

impl Record for Entity {
    type QueryPath<'p> = EntityQueryPath<'p>;
    type VertexId = EntityVertexId;

    #[must_use]
    fn vertex_id(&self, time_axis: TimeAxis) -> EntityVertexId {
        let ClosedTemporalBound::Inclusive(timestamp) = match time_axis {
            TimeAxis::DecisionTime => self
                .metadata
                .temporal_versioning()
                .decision_time
                .start()
                .cast(),
            TimeAxis::TransactionTime => self
                .metadata
                .temporal_versioning()
                .transaction_time
                .start()
                .cast(),
        };
        EntityVertexId {
            base_id: self.metadata.record_id().entity_id,
            revision_id: timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, iter::once};

    use super::*;

    fn deserialize<'p>(segments: impl IntoIterator<Item = &'p str>) -> EntityQueryPath<'p> {
        EntityQueryPath::deserialize(de::value::SeqDeserializer::<_, de::value::Error>::new(
            segments.into_iter(),
        ))
        .expect("could not deserialize path")
    }

    #[test]
    fn deserialization() {
        assert_eq!(deserialize(["ownedById"]), EntityQueryPath::OwnedById);
        assert_eq!(
            deserialize(["type", "version"]),
            EntityQueryPath::EntityTypeEdge {
                edge_kind: SharedEdgeKind::IsOfType,
                path: EntityTypeQueryPath::Version,
                inheritance_depth: None,
            }
        );
        assert_eq!(
            deserialize(["type(inheritanceDepth = 5)", "version"]),
            EntityQueryPath::EntityTypeEdge {
                edge_kind: SharedEdgeKind::IsOfType,
                path: EntityTypeQueryPath::Version,
                inheritance_depth: Some(5),
            }
        );
        assert_eq!(
            deserialize([
                "properties",
                "https://blockprotocol.org/@alice/types/property-type/name/"
            ]),
            EntityQueryPath::Properties(Some(JsonPath::from_path_tokens(vec![PathToken::Field(
                Cow::Borrowed("https://blockprotocol.org/@alice/types/property-type/name/")
            )])))
        );
        assert_eq!(
            deserialize(["leftEntity", "uuid"]),
            EntityQueryPath::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
                path: Box::new(EntityQueryPath::Uuid),
                direction: EdgeDirection::Outgoing
            }
        );

        assert_eq!(
            EntityQueryPath::deserialize(de::value::SeqDeserializer::<_, de::value::Error>::new(
                once("invalid")
            ))
            .expect_err(
                "managed to convert entity query path with hidden token when it should have \
                 errored"
            )
            .to_string(),
            format!(
                "unknown variant `invalid`, expected {}",
                EntityQueryPathVisitor::EXPECTING
            )
        );

        assert_eq!(
            EntityQueryPath::deserialize(de::value::SeqDeserializer::<_, de::value::Error>::new(
                ["editionId", "test"].into_iter()
            ))
            .expect_err(
                "managed to convert entity query path with multiple tokens when it should have \
                 errored"
            )
            .to_string(),
            "invalid length 2, expected 1 element in sequence"
        );
    }
}
