use async_trait::async_trait;
use error_stack::{IntoReport, Report, Result, ResultExt};
use tokio_postgres::GenericClient;

use crate::{
    knowledge::{EntityId, Link, Links, Outgoing},
    ontology::types::uri::VersionedUri,
    store::{crud, AsClient, PostgresStore, QueryError},
};

#[async_trait]
impl<C: AsClient> crud::Read<'_, EntityId, Link> for PostgresStore<C> {
    type Output = Links;

    async fn get(&self, identifier: EntityId) -> Result<Self::Output, QueryError> {
        let multi_links = self
            .client
            .as_client()
            .query(
                r#"
                WITH aggregated as (
                    SELECT link_type_version_id, ARRAY_AGG(target_entity_id ORDER BY multi_order ASC) as links
                    FROM links
                    WHERE active AND multi and source_entity_id = $1
                    GROUP BY link_type_version_id
                )
                SELECT base_uri, "version", links FROM aggregated
                INNER JOIN ids ON ids.version_id = aggregated.link_type_version_id
                "#,
                &[&identifier],
            )
            .await
            .report()
            .change_context(QueryError)
            .attach_printable(identifier)?
            .into_iter()
            .map(|row| (VersionedUri::new(row.get(0), row.get::<_, i64>(1) as u32), Outgoing::Multiple(row.get(2))));

        let single_links = self
            .client
            .as_client()
            .query(
                r#"
                SELECT base_uri, "version", target_entity_id
                FROM links
                INNER JOIN ids ON ids.version_id = links.link_type_version_id
                WHERE active AND NOT multi and source_entity_id = $1
                "#,
                &[&identifier],
            )
            .await
            .report()
            .change_context(QueryError)
            .attach_printable(identifier)?
            .into_iter()
            .map(|row| {
                (
                    VersionedUri::new(row.get(0), row.get::<_, i64>(1) as u32),
                    Outgoing::Single(row.get(2)),
                )
            });

        Ok(Links::new(multi_links.chain(single_links).collect()))
    }
}

#[async_trait]
impl<'i, C: AsClient> crud::Read<'i, (EntityId, &'i VersionedUri), Link> for PostgresStore<C> {
    type Output = Outgoing;

    async fn get(
        &self,
        identifier: (EntityId, &'i VersionedUri),
    ) -> Result<Self::Output, QueryError> {
        let (source_entity_id, link_type_uri) = identifier;
        let version = i64::from(link_type_uri.version());
        let link = self
        .client
        .as_client()
        .query_one(
            r#"
                -- Gather all single-links
                WITH single_links AS (
                    SELECT link_type_version_id, target_entity_id
                    FROM links
                    INNER JOIN ids ON ids.version_id = links.link_type_version_id
                    WHERE active AND NOT multi AND source_entity_id = $1 AND base_uri = $2 AND "version" = $3
                ),
                -- Gather all multi-links
                multi_links AS (
                    SELECT link_type_version_id, ARRAY_AGG(target_entity_id ORDER BY multi_order ASC) AS target_entity_ids
                    FROM links
                    INNER JOIN ids ON ids.version_id = links.link_type_version_id
                    WHERE active AND multi AND source_entity_id = $1 AND base_uri = $2 AND "version" = $3
                    GROUP BY link_type_version_id
                )
                -- Combine single and multi links with null values in rows where the other doesn't exist
                SELECT link_type_version_id, target_entity_id AS single_link, NULL AS multi_link FROM single_links 
                UNION 
                SELECT link_type_version_id, NULL AS single_link, target_entity_ids AS multi_link from multi_links
                "#,
            &[&source_entity_id, link_type_uri.base_uri(), &version],
        )
        .await
        .report()
        .change_context(QueryError)
        .attach_printable(source_entity_id)
        .attach_printable(link_type_uri.clone())?;

        let val: (Option<EntityId>, Option<Vec<EntityId>>) = (link.get(1), link.get(2));
        match val {
            (Some(entity_id), None) => Ok(Outgoing::Single(entity_id)),
            (None, Some(entity_ids)) => Ok(Outgoing::Multiple(entity_ids)),
            _ => Err(Report::new(QueryError)
                .attach_printable(source_entity_id)
                .attach_printable(link_type_uri.clone())),
        }
    }
}
