import { useState, useEffect, useCallback } from 'react';
import { AuthClient } from '@dfinity/auth-client';
import { HttpAgent } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';

const II_URL = 'https://identity.internetcomputer.org';
const LOCAL_II_CANISTER_ID = 'rwlgt-iiaaa-aaaaa-aaaaa-cai';

export interface AuthState {
  isAuthenticated: boolean;
  principal: Principal | null;
  agent: HttpAgent | null;
  authClient: AuthClient | null;
  isLoading: boolean;
  error: string | null;
}

export function useAuth() {
  const [state, setState] = useState<AuthState>({
    isAuthenticated: false,
    principal: null,
    agent: null,
    authClient: null,
    isLoading: true,
    error: null,
  });

  const initAuth = useCallback(async () => {
    try {
      const authClient = await AuthClient.create();
      const identity = authClient.getIdentity();
      const principal = identity.getPrincipal();

      const isLocal = window.location.hostname === 'localhost' ||
                      window.location.hostname === '127.0.0.1';

      const agent = new HttpAgent({
        identity,
        host: isLocal ? 'http://127.0.0.1:8000' : 'https://icp-api.io',
      });

      if (isLocal) {
        await agent.fetchRootKey();
      }

      const isAuthenticated = !principal.isAnonymous();

      setState({
        isAuthenticated,
        principal: isAuthenticated ? principal : null,
        agent,
        authClient,
        isLoading: false,
        error: null,
      });
    } catch (err) {
      setState(prev => ({
        ...prev,
        isLoading: false,
        error: err instanceof Error ? err.message : 'Auth initialization failed',
      }));
    }
  }, []);

  useEffect(() => {
    initAuth();
  }, [initAuth]);

  const login = useCallback(async () => {
    setState(prev => ({ ...prev, isLoading: true, error: null }));

    const isLocal = window.location.hostname === 'localhost' ||
                    window.location.hostname === '127.0.0.1';

    const authClient = await AuthClient.create();

    return new Promise<void>((resolve, reject) => {
      authClient.login({
        identityProvider: isLocal ? `http://${window.location.hostname}:8000?canisterId=${LOCAL_II_CANISTER_ID}` : II_URL,
        onSuccess: async () => {
          const identity = authClient.getIdentity();
          const principal = identity.getPrincipal();

          const agent = new HttpAgent({
            identity,
            host: isLocal ? 'http://127.0.0.1:8000' : 'https://icp-api.io',
          });

          if (isLocal) {
            await agent.fetchRootKey();
          }

          setState({
            isAuthenticated: true,
            principal,
            agent,
            authClient,
            isLoading: false,
            error: null,
          });
          resolve();
        },
        onError: (err) => {
          setState(prev => ({
            ...prev,
            isLoading: false,
            error: err instanceof Error ? err.message : 'Login failed',
          }));
          reject(err);
        },
      });
    });
  }, []);

  const logout = useCallback(async () => {
    if (state.authClient) {
      await state.authClient.logout();
    }
    setState({
      isAuthenticated: false,
      principal: null,
      agent: null,
      authClient: null,
      isLoading: false,
      error: null,
    });
  }, [state.authClient]);

  return {
    ...state,
    login,
    logout,
    refreshAuth: initAuth,
  };
}