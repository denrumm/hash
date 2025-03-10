mod filter;
mod path;

use std::{collections::HashMap, fmt};

use serde::{
    de::{self, IntoDeserializer},
    Deserialize,
};

pub use self::{
    filter::{Filter, FilterExpression, Parameter, ParameterConversionError, ParameterList},
    path::{JsonPath, PathToken},
};

pub trait QueryPath {
    /// Returns what type this resolved `Path` has.
    fn expected_type(&self) -> ParameterType;
}

/// Parses a query token of the form `token(key=value)`.
///
/// Whitespaces are ignored and multiple parameters are supported.
///
/// # Errors
///
/// - If the token is not of the form `token`, `token()`, or `token(key=value)`
/// - If `token` can not be deserialized into `T`
pub fn parse_query_token<'de, T: Deserialize<'de>, E: de::Error>(
    token: &'de str,
) -> Result<(T, HashMap<&str, &str>), E> {
    let Some((token, parameters)) = token.split_once('(') else {
        return T::deserialize(token.into_deserializer()).map(|token| (token, HashMap::new()));
    };

    let parameters = parameters
        .strip_suffix(')')
        .ok_or_else(|| E::custom("missing closing parenthesis"))?
        .split(',')
        .filter(|parameter| !parameter.trim().is_empty())
        .map(|parameter| {
            let (key, value) = parameter
                .split_once('=')
                .ok_or_else(|| E::custom("missing parameter value, expected `key=value`"))?;
            Ok((key.trim(), value.trim()))
        })
        .collect::<Result<_, _>>()?;

    T::deserialize(token.into_deserializer()).map(|token| (token, parameters))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ParameterType {
    Boolean,
    Number,
    OntologyTypeVersion,
    Text,
    Uuid,
    BaseUrl,
    VersionedUrl,
    TimeInterval,
    Timestamp,
    Object,
    Any,
}

impl fmt::Display for ParameterType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean => fmt.write_str("boolean"),
            Self::Number => fmt.write_str("number"),
            Self::OntologyTypeVersion => fmt.write_str("ontology type version"),
            Self::Text => fmt.write_str("text"),
            Self::Uuid => fmt.write_str("UUID"),
            Self::BaseUrl => fmt.write_str("base URL"),
            Self::VersionedUrl => fmt.write_str("versioned URL"),
            Self::TimeInterval => fmt.write_str("time interval"),
            Self::Timestamp => fmt.write_str("timestamp"),
            Self::Object => fmt.write_str("object"),
            Self::Any => fmt.write_str("any"),
        }
    }
}

pub trait OntologyQueryPath {
    /// Returns the path identifying the internal ontology id.
    fn ontology_id() -> Self;

    /// Returns the path identifying the [`BaseUrl`].
    ///
    /// [`BaseUrl`]: type_system::url::BaseUrl
    fn base_url() -> Self;

    /// Returns the path identifying the [`VersionedUrl`].
    ///
    /// [`VersionedUrl`]: type_system::url::VersionedUrl
    fn versioned_url() -> Self;

    /// Returns the path identifying the [`OntologyTypeVersion`].
    ///
    /// [`OntologyTypeVersion`]: graph_types::ontology::OntologyTypeVersion
    fn version() -> Self;

    /// Returns the path identifying the transaction time.
    fn transaction_time() -> Self;

    /// Returns the path identifying the [`RecordCreatedById`].
    ///
    /// [`RecordCreatedById`]: graph_types::provenance::RecordCreatedById
    fn record_created_by_id() -> Self;

    /// Returns the path identifying the [`RecordArchivedById`].
    ///
    /// [`RecordArchivedById`]: graph_types::provenance::RecordArchivedById
    fn record_archived_by_id() -> Self;

    /// Returns the path identifying the schema.
    fn schema() -> Self;

    /// Returns the path identifying the metadata
    fn additional_metadata() -> Self;
}
