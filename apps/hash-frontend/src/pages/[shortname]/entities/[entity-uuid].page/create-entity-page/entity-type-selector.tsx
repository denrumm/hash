import { EntityType } from "@blockprotocol/type-system";
import {
  EntityTypeIcon,
  LinkTypeIcon,
  SelectorAutocomplete,
} from "@hashintel/design-system";
import { EntityTypeWithMetadata } from "@local/hash-subgraph";
import { FunctionComponent, useRef, useState } from "react";

import { useEntityTypesContextRequired } from "../../../../../shared/entity-types-context/hooks/use-entity-types-context-required";

export const EntityTypeSelector: FunctionComponent<{
  onSelect: (entityType: EntityType) => void;
  onCancel: () => void;
  onCreateNew: (searchValue: string) => void;
}> = ({ onCancel, onSelect, onCreateNew }) => {
  const [search, setSearch] = useState("");
  const { entityTypes, isLinkTypeLookup } = useEntityTypesContextRequired();

  const [open, setOpen] = useState(false);
  const highlightedRef = useRef<null | EntityTypeWithMetadata>(null);

  return (
    <SelectorAutocomplete
      dropdownProps={{
        query: search,
        createButtonProps: {
          onMouseDown: (evt) => {
            evt.preventDefault();
            evt.stopPropagation();
            onCreateNew(search);
          },
        },
        variant: "entity type",
      }}
      options={entityTypes ?? []}
      optionToRenderData={({ schema: { $id, title, description } }) => ({
        uniqueId: $id,
        Icon: isLinkTypeLookup?.[$id] ? LinkTypeIcon : EntityTypeIcon,
        typeId: $id,
        title,
        description,
      })}
      inputPlaceholder="Search for an entity type"
      open={open}
      onOpen={() => setOpen(true)}
      onClose={(_, reason) => {
        if (reason !== "toggleInput") {
          setOpen(false);
        }
      }}
      inputValue={search}
      onInputChange={(_, value) => setSearch(value)}
      onHighlightChange={(_, value) => {
        highlightedRef.current = value;
      }}
      onChange={(_, option) => {
        onSelect(option.schema);
      }}
      onKeyUp={(evt) => {
        if (evt.key === "Enter" && !highlightedRef.current) {
          onCreateNew(search);
        }
      }}
      onKeyDown={(evt) => {
        if (evt.key === "Escape") {
          onCancel();
        }
      }}
      onBlur={() => {
        onCancel();
      }}
      sx={{ maxWidth: 440 }}
    />
  );
};
