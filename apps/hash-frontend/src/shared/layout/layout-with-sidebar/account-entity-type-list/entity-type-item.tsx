import { VersionedUrl } from "@blockprotocol/type-system";
import { IconButton } from "@hashintel/design-system";
import { extractBaseUrl } from "@local/hash-subgraph/type-system-patch";
import { Box, BoxProps, styled, Tooltip, Typography } from "@mui/material";
import { bindTrigger, usePopupState } from "material-ui-popup-state/hooks";
import { useRouter } from "next/router";
import { FunctionComponent, useRef } from "react";

import { EllipsisRegularIcon } from "../../../icons/ellipsis-regular-icon";
import { Link } from "../../../ui";
import { EntityTypeMenu } from "./entity-type-menu";

type EntityTypeItemProps = {
  entityTypeId: VersionedUrl;
  title: string;
};

const Container = styled((props: BoxProps & { selected: boolean }) => (
  <Box component="li" {...props} />
))(({ theme, selected }) => ({
  paddingLeft: theme.spacing(2),
  paddingRight: theme.spacing(1),
  paddingTop: theme.spacing(0.5),
  paddingBottom: theme.spacing(0.5),
  margin: `0 ${theme.spacing(0.5)}`,
  borderRadius: "4px",
  display: "flex",
  justifyContent: "space-between",
  alignItems: "center",
  color: theme.palette.gray[70],

  "&:hover, &:focus": {
    backgroundColor: selected ? theme.palette.gray[20] : theme.palette.gray[10],
    color: selected ? theme.palette.gray[80] : theme.palette.gray[70],

    "& .entity-menu-trigger": {
      color: theme.palette.gray[80],
    },
  },

  ...(selected && {
    backgroundColor: theme.palette.gray[30],
    color: theme.palette.gray[80],
  }),

  "&:focus-within": {
    backgroundColor: theme.palette.gray[20],
    color: theme.palette.gray[80],
  },
}));

export const EntityTypeItem: FunctionComponent<EntityTypeItemProps> = ({
  entityTypeId,
  title,
}) => {
  const entityMenuTriggerRef = useRef(null);
  const popupState = usePopupState({
    variant: "popover",
    popupId: "entity-menu",
  });

  const router = useRouter();

  // @todo once data fetching is moved to server, use that to identify if selected
  const baseUrl = extractBaseUrl(entityTypeId);
  const url = new URL(`${window.location.origin}${router.asPath}/`);
  const urlBase = `${url.origin}${url.pathname.replace(/\/$/, "")}/`;
  const selected =
    router.route === "/[shortname]/types/entity-type/[entity-type-id]" &&
    urlBase === baseUrl;

  return (
    <Container component="li" tabIndex={0} selected={selected}>
      <Link tabIndex={-1} sx={{ flex: 1 }} noLinkStyle href={baseUrl} flex={1}>
        <Typography
          variant="smallTextLabels"
          sx={{
            display: "block",
            color: ({ palette }) => palette.gray[70],
            fontWeight: 500,
          }}
        >
          {title}
        </Typography>
      </Link>
      <Tooltip title="Options" sx={{ left: 5 }}>
        <IconButton
          ref={entityMenuTriggerRef}
          className="entity-menu-trigger"
          {...bindTrigger(popupState)}
          size="medium"
          unpadded
          sx={({ palette }) => ({
            color: [selected ? palette.gray[80] : "transparent"],
            "&:focus-visible, &:hover": {
              backgroundColor: palette.gray[selected ? 40 : 30],
              color: palette.gray[selected ? 80 : 40],
            },
          })}
        >
          <EllipsisRegularIcon />
        </IconButton>
      </Tooltip>
      <EntityTypeMenu
        entityTypeId={entityTypeId}
        popupState={popupState}
        title={title}
        url={baseUrl}
      />
    </Container>
  );
};
