import { gql } from "@apollo/client";
import { commentFieldsFragment } from "./comment.queries";

export const setParentPage = gql`
  mutation setParentPage(
    $pageEntityId: ID!
    $parentPageEntityId: ID
    $prevIndex: String
    $nextIndex: String
  ) {
    setParentPersistedPage(
      pageEntityId: $pageEntityId
      parentPageEntityId: $parentPageEntityId
      prevIndex: $prevIndex
      nextIndex: $nextIndex
    ) {
      metadata
      title
      summary
      __typename
    }
  }
`;

export const createPersistedPage = gql`
  mutation createPersistedPage(
    $ownedById: ID!
    $properties: PersistedPageCreationData!
  ) {
    createPersistedPage(ownedById: $ownedById, properties: $properties) {
      metadata
    }
  }
`;

export const updatePersistedPage = gql`
  mutation updatePersistedPage(
    $entityId: ID!
    $updatedProperties: PersistedPageUpdateData!
  ) {
    updatePersistedPage(
      entityId: $entityId
      updatedProperties: $updatedProperties
    ) {
      metadata
    }
  }
`;

export const getPersistedPageComments = gql`
  query getPersistedPageComments($entityId: ID!) {
    persistedPageComments(entityId: $entityId) {
      ...CommentFields
      replies {
        ...CommentFields
      }
    }
  }
  ${commentFieldsFragment}
`;
