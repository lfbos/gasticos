/**
 * Zustand store for authentication state.
 */

import { create } from 'zustand';
import { storage } from '@/lib/storage';
import type { User } from '@/types';

interface AuthState {
  user: User | null;
  accessToken: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

interface AuthActions {
  setAuth: (user: User, accessToken: string, refreshToken: string) => void;
  setUser: (user: User) => void;
  setLoading: (isLoading: boolean) => void;
  logout: () => void;
  initialize: () => void;
}

// Check for tokens on store creation (synchronous)
const initialAccessToken = typeof window !== 'undefined' ? localStorage.getItem('gasticos_access_token') : null;
const initialRefreshToken = typeof window !== 'undefined' ? localStorage.getItem('gasticos_refresh_token') : null;
const hasTokens = !!(initialAccessToken && initialRefreshToken);

export const useAuthStore = create<AuthState & AuthActions>((set) => ({
  user: null,
  accessToken: initialAccessToken,
  refreshToken: initialRefreshToken,
  isAuthenticated: hasTokens,
  isLoading: hasTokens, // Only loading if we need to fetch user

  setAuth: (user, accessToken, refreshToken) => {
    storage.setTokens(accessToken, refreshToken);
    set({
      user,
      accessToken,
      refreshToken,
      isAuthenticated: true,
      isLoading: false,
    });
  },

  setUser: (user) => {
    set({ user });
  },

  setLoading: (isLoading) => {
    set({ isLoading });
  },

  logout: () => {
    storage.clearTokens();
    set({
      user: null,
      accessToken: null,
      refreshToken: null,
      isAuthenticated: false,
      isLoading: false,
    });
  },

  initialize: () => {
    const accessToken = storage.getAccessToken();
    const refreshToken = storage.getRefreshToken();

    if (accessToken && refreshToken) {
      set({
        accessToken,
        refreshToken,
        isAuthenticated: true,
        isLoading: true,
      });
    } else {
      set({ isLoading: false });
    }
  },
}));
