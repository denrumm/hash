import { extractVersion, VersionedUrl } from "@blockprotocol/type-system";
import { AsteriskRegularIcon } from "@hashintel/design-system";
import { types } from "@local/hash-isomorphic-utils/ontology-types";
import { isBaseUrl, OwnedById } from "@local/hash-subgraph";
import { extractBaseUrl } from "@local/hash-subgraph/type-system-patch";
import {
  Box,
  buttonClasses,
  Container,
  Typography,
  useTheme,
} from "@mui/material";
import { useRouter } from "next/router";
import { NextSeo } from "next-seo";
import { useCallback, useContext, useMemo } from "react";

import { useAccountPages } from "../components/hooks/use-account-pages";
import { useCreatePage } from "../components/hooks/use-create-page";
import { EntityTypeEntitiesContext } from "../shared/entity-type-entities-context";
import { useEntityTypeEntitiesContextValue } from "../shared/entity-type-entities-context/use-entity-type-entities-context-value";
import { useLatestEntityTypesOptional } from "../shared/entity-types-context/hooks";
import { AsteriskLightIcon } from "../shared/icons/asterisk-light-icon";
import { FileCirclePlusRegularIcon } from "../shared/icons/file-circle-plus-regular-icon";
import { FilesLightIcon } from "../shared/icons/files-light-icon";
import { FilesRegularIcon } from "../shared/icons/files-regular-icon";
import { getLayoutWithSidebar, NextPageWithLayout } from "../shared/layout";
import { HEADER_HEIGHT } from "../shared/layout/layout-with-header/page-header";
import { tableHeaderHeight } from "../shared/table-header";
import { Button } from "../shared/ui";
import { TabLink } from "../shared/ui/tab-link";
import { Tabs } from "../shared/ui/tabs";
import { EntitiesTable } from "./shared/entities-table";
import {
  TOP_CONTEXT_BAR_HEIGHT,
  TopContextBar,
} from "./shared/top-context-bar";
import { WorkspaceContext } from "./shared/workspace-context";

const contentMaxWidth = 1000;

type ParsedQueryParams = {
  entityTypeIdOrBaseUrl?: string;
};

const EntitiesPage: NextPageWithLayout = () => {
  const router = useRouter();
  const { activeWorkspace } = useContext(WorkspaceContext);

  const { entityTypeId, entityTypeBaseUrl } = useMemo(() => {
    if (router.isReady) {
      const { entityTypeIdOrBaseUrl } = router.query as ParsedQueryParams;

      return {
        entityTypeId:
          entityTypeIdOrBaseUrl && !isBaseUrl(entityTypeIdOrBaseUrl)
            ? (entityTypeIdOrBaseUrl as VersionedUrl)
            : undefined,
        entityTypeBaseUrl:
          entityTypeIdOrBaseUrl && isBaseUrl(entityTypeIdOrBaseUrl)
            ? entityTypeIdOrBaseUrl
            : undefined,
      };
    }
    return {};
  }, [router]);

  const { lastRootPageIndex } = useAccountPages(
    activeWorkspace?.accountId as OwnedById,
  );
  const [createUntitledPage] = useCreatePage(
    activeWorkspace?.accountId as OwnedById,
  );

  const createPage = useCallback(async () => {
    await createUntitledPage(lastRootPageIndex);
  }, [lastRootPageIndex, createUntitledPage]);

  const latestEntityTypes = useLatestEntityTypesOptional({
    includeArchived: true,
  });

  const entityType = useMemo(
    () =>
      entityTypeId || entityTypeBaseUrl
        ? latestEntityTypes?.find(({ schema }) =>
            entityTypeId
              ? schema.$id === entityTypeId
              : extractBaseUrl(schema.$id) === entityTypeBaseUrl,
          )
        : undefined,
    [latestEntityTypes, entityTypeId, entityTypeBaseUrl],
  );

  const entityTypeEntitiesValue = useEntityTypeEntitiesContextValue({
    entityTypeBaseUrl,
    entityTypeId,
  });

  const isViewAllPagesPage =
    entityType?.schema.$id === types.entityType.page.entityTypeId;

  const pageTitle = entityType
    ? entityTypeId
      ? `${entityType.schema.title} v${extractVersion(entityTypeId)}`
      : `${entityType.schema.title}s`
    : "Entities";

  const theme = useTheme();

  const { entities, loading } = entityTypeEntitiesValue;

  return (
    <>
      <NextSeo title={pageTitle} />
      <TopContextBar
        defaultCrumbIcon={null}
        crumbs={[
          {
            title: pageTitle,
            href: isViewAllPagesPage
              ? "/pages"
              : `/entities${
                  entityTypeId ? `?entityTypeIdOrBaseUrl=${entityTypeId}` : ""
                }`,
            id: entityTypeId ?? "entities",
            icon: isViewAllPagesPage ? (
              <FilesRegularIcon />
            ) : (
              <AsteriskRegularIcon />
            ),
          },
        ]}
        scrollToTop={() => {}}
      />
      <Box
        sx={{
          borderBottom: 1,
          borderColor: ({ palette }) => palette.gray[20],
          pt: 3.75,
          backgroundColor: ({ palette }) => palette.common.white,
        }}
      >
        <Container sx={{ maxWidth: { lg: contentMaxWidth } }}>
          <Typography variant="h1" fontWeight="bold" my={3}>
            <Box
              display="inline-flex"
              sx={({ palette }) => ({
                svg: {
                  fontSize: 40,
                  mr: 2,
                  color: palette.gray[70],
                  verticalAlign: "middle",
                },
              })}
            >
              {isViewAllPagesPage ? <FilesLightIcon /> : <AsteriskLightIcon />}
            </Box>
            {pageTitle}
          </Typography>
          <Box display="flex" justifyContent="space-between">
            <Tabs value="all">
              <TabLink
                href="/"
                value="all"
                label={`All ${pageTitle}`}
                count={entities?.length}
                loading={loading}
              />
            </Tabs>
            {isViewAllPagesPage ? (
              <Button
                endIcon={<FileCirclePlusRegularIcon />}
                variant="tertiary_quiet"
                onClick={createPage}
                sx={{
                  color: ({ palette }) => palette.gray[90],
                  fontSize: 14,
                  padding: 0,
                  ":hover": {
                    background: "transparent",
                    color: ({ palette }) => palette.blue[70],
                    [`.${buttonClasses.endIcon}`]: {
                      color: ({ palette }) => palette.blue[70],
                    },
                  },
                  [`.${buttonClasses.endIcon}`]: {
                    color: ({ palette }) => palette.blue[70],
                  },
                }}
              >
                Create a page
              </Button>
            ) : null}
          </Box>
        </Container>
      </Box>
      <Container sx={{ maxWidth: { lg: contentMaxWidth }, py: 5 }}>
        <EntityTypeEntitiesContext.Provider value={entityTypeEntitiesValue}>
          <EntitiesTable
            hideEntityTypeVersionColumn={!!entityTypeId}
            hidePropertiesColumns
            height={
              entities && entities.length > 10
                ? `calc(100vh - (${
                    HEADER_HEIGHT +
                    TOP_CONTEXT_BAR_HEIGHT +
                    179 +
                    tableHeaderHeight
                  }px + ${theme.spacing(5)} + ${theme.spacing(5)}))`
                : undefined
            }
          />
        </EntityTypeEntitiesContext.Provider>
      </Container>
    </>
  );
};

EntitiesPage.getLayout = (page) =>
  getLayoutWithSidebar(page, {
    fullWidth: true,
  });

export default EntitiesPage;
