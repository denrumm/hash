import { VersionedUrl } from "@blockprotocol/type-system/dist/cjs";
import { EntityType } from "@blockprotocol/type-system/slim";
import { BaseUrl, EntityTypeWithMetadata } from "@local/hash-subgraph";
import { useMemo } from "react";

import { useEntityTypesContextRequired } from "./hooks/use-entity-types-context-required";
import { isLinkEntityType } from "./shared/is-link-entity-type";
import { isTypeArchived } from "./util";

export const useEntityTypesLoading = () =>
  useEntityTypesContextRequired().loading;

export const useEntityTypesOptional = (params?: {
  includeArchived?: boolean;
}) => {
  const { includeArchived = false } = params ?? {};

  const { entityTypes } = useEntityTypesContextRequired();

  return includeArchived
    ? entityTypes
    : entityTypes?.filter((entityType) => !isTypeArchived(entityType));
};

export const useEntityTypesSubgraphOptional = () =>
  useEntityTypesContextRequired().subgraph;

export const useFetchEntityTypes = () =>
  useEntityTypesContextRequired().refetch;

export const useLatestEntityTypesOptional = (params?: {
  includeArchived: boolean;
}) => {
  const { includeArchived = false } = params ?? {};

  const entityTypes = useEntityTypesOptional({ includeArchived });

  return useMemo(() => {
    if (!entityTypes) {
      return null;
    }

    const latestEntityTypes: Map<BaseUrl, EntityTypeWithMetadata> = new Map();

    for (const entityType of entityTypes) {
      const baseUrl = entityType.metadata.recordId.baseUrl;

      const existingEntityType = latestEntityTypes.get(baseUrl);
      if (
        !existingEntityType ||
        existingEntityType.metadata.recordId.version <
          entityType.metadata.recordId.version
      ) {
        latestEntityTypes.set(baseUrl, entityType);
      }
    }

    return Array.from(latestEntityTypes.values());
  }, [entityTypes]);
};

/**
 * Check if a specific entity type is or would be a link type, based on the provided 'allOf'
 * Specifically for use for checking types which aren't already in the db, e.g. draft or proposed types
 *
 * For types already in the db, do this instead:
 *   const { isLinkTypeLookup } = useEntityTypesContextRequired();
 *   const isLinkType = isLinkTypeLookup?.[entityType.$id];
 */
export const useIsLinkType = (entityType: Pick<EntityType, "allOf">) => {
  const entityTypes = useEntityTypesOptional({ includeArchived: true });

  return useMemo(() => {
    const typesByVersion: Record<VersionedUrl, EntityTypeWithMetadata> =
      Object.fromEntries(
        (entityTypes ?? []).map((type) => [type.schema.$id, type]),
      );

    return isLinkEntityType(entityType, typesByVersion);
  }, [entityType, entityTypes]);
};
