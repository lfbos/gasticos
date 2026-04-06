/**
 * Authentication hook combining Zustand store with API mutations.
 * DOM-free for React Native reuse.
 */

import { useCallback, useEffect } from 'react';
import { useAuthStore } from '@/stores/authStore';
import { authApi, getApiErrorMessage } from '@/api';
import type { LoginRequest, RegisterRequest, User } from '@/types';

interface UseAuthReturn {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (data: LoginRequest) => Promise<void>;
  register: (data: RegisterRequest) => Promise<void>;
  logout: () => Promise<void>;
  fetchUser: () => Promise<void>;
}

export function useAuth(): UseAuthReturn {
  const {
    user,
    isAuthenticated,
    isLoading,
    setAuth,
    setUser,
    setLoading,
    logout: clearAuth,
  } = useAuthStore();

  // Fetch user data if authenticated but user is not loaded
  useEffect(() => {
    if (isAuthenticated && !user) {
      setLoading(true);
      authApi
        .me()
        .then((userData) => {
          setUser(userData);
          setLoading(false);
        })
        .catch(() => {
          clearAuth();
        });
    }
  }, [isAuthenticated, user, setUser, setLoading, clearAuth]);

  const login = useCallback(
    async (data: LoginRequest) => {
      try {
        const response = await authApi.login(data);
        setAuth(response.user, response.access_token, response.refresh_token);
      } catch (error) {
        throw new Error(getApiErrorMessage(error));
      }
    },
    [setAuth]
  );

  const register = useCallback(
    async (data: RegisterRequest) => {
      try {
        const response = await authApi.register(data);
        setAuth(response.user, response.access_token, response.refresh_token);
      } catch (error) {
        throw new Error(getApiErrorMessage(error));
      }
    },
    [setAuth]
  );

  const logout = useCallback(async () => {
    try {
      await authApi.logout();
    } catch {
      // Ignore errors on logout
    } finally {
      clearAuth();
    }
  }, [clearAuth]);

  const fetchUser = useCallback(async () => {
    try {
      const userData = await authApi.me();
      setUser(userData);
    } catch (error) {
      throw new Error(getApiErrorMessage(error));
    }
  }, [setUser]);

  return {
    user,
    isAuthenticated,
    isLoading,
    login,
    register,
    logout,
    fetchUser,
  };
}
