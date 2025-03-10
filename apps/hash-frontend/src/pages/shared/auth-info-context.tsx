import { useLazyQuery } from "@apollo/client";
import { UserProperties } from "@local/hash-isomorphic-utils/system-types/shared";
import { Entity, EntityRootType, Subgraph } from "@local/hash-subgraph";
import { getRoots } from "@local/hash-subgraph/stdlib";
import {
  createContext,
  FunctionComponent,
  ReactElement,
  useCallback,
  useContext,
  useMemo,
  useState,
} from "react";

import { MeQuery } from "../../graphql/api-types.gen";
import { meQuery } from "../../graphql/queries/user.queries";
import {
  AuthenticatedUser,
  constructAuthenticatedUser,
} from "../../lib/user-and-org";
import { fetchKratosSession } from "./ory-kratos";

type RefetchAuthInfoFunction = () => Promise<{
  authenticatedUser?: AuthenticatedUser;
}>;

type AuthInfoState = {
  authenticatedUser?: AuthenticatedUser;
  refetch: RefetchAuthInfoFunction;
};

export const AuthInfoContext = createContext<AuthInfoState | undefined>(
  undefined,
);

type AuthInfoProviderProps = {
  initialAuthenticatedUser?: AuthenticatedUser;
  children: ReactElement;
};

export const AuthInfoProvider: FunctionComponent<AuthInfoProviderProps> = ({
  initialAuthenticatedUser,
  children,
}) => {
  const [authenticatedUser, setAuthenticatedUser] = useState<
    AuthenticatedUser | undefined
  >(initialAuthenticatedUser); // use the initial server-sent data to start – after that, the client controls the value

  const [getMe] = useLazyQuery<MeQuery>(meQuery, { fetchPolicy: "no-cache" });

  const fetchAuthenticatedUser =
    useCallback<RefetchAuthInfoFunction>(async () => {
      const [subgraph, kratosSession] = await Promise.all([
        getMe()
          .then(({ data }) => data?.me)
          .catch(() => undefined),
        fetchKratosSession(),
      ]);

      if (!subgraph || !kratosSession) {
        setAuthenticatedUser(undefined);
        return {};
      }
      const userEntity = getRoots(subgraph as Subgraph<EntityRootType>)[0]!;

      const latestAuthenticatedUser = constructAuthenticatedUser({
        userEntity: userEntity as Entity<UserProperties>,
        subgraph,
        kratosSession,
      });

      setAuthenticatedUser(latestAuthenticatedUser);

      return {
        authenticatedUser: latestAuthenticatedUser,
      };
    }, [getMe]);

  const value = useMemo(
    () => ({
      authenticatedUser,
      refetch: fetchAuthenticatedUser,
    }),
    [authenticatedUser, fetchAuthenticatedUser],
  );

  return (
    <AuthInfoContext.Provider value={value}>
      {children}
    </AuthInfoContext.Provider>
  );
};

export const useAuthInfo = (): AuthInfoState => {
  const authInfo = useContext(AuthInfoContext);

  if (!authInfo) {
    throw new Error(
      "Cannot use auth info because it has not been defined. Verify `useAuthInfo` is being called within a child of the `AuthInfoProvider`.",
    );
  }

  return authInfo;
};

/**
 * Use the currently authenticated user.
 *
 * @throws if there is no user authenticated in the application.
 * @returns `AuthInfo` where the `authenticatedUser` is always defined.
 */
export const useAuthenticatedUser = (): AuthInfoState & {
  authenticatedUser: AuthenticatedUser;
} => {
  const { authenticatedUser, ...remainingAuthInfo } = useAuthInfo();

  if (!authenticatedUser) {
    throw new Error(
      "There is no authenticated user. Consider using the `useAuthInfo` hook instead if this is expected.",
    );
  }

  return { authenticatedUser, ...remainingAuthInfo };
};
