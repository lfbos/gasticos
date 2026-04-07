/**
 * Belvo Open Banking hook.
 * DOM-free for React Native reuse.
 */

import { useCallback, useEffect, useState } from 'react';
import { belvoApi } from '@/api/belvo';
import { getApiErrorMessage } from '@/api/client';
import type {
  BelvoAccount,
  BelvoWidgetSuccessEvent,
  ConnectedBank,
} from '@/types/belvo';

interface UseBelvoReturn {
  // State
  connectedBanks: ConnectedBank[];
  accounts: BelvoAccount[];
  isLoading: boolean;
  isSyncing: Record<string, boolean>;
  error: string | null;
  widgetToken: string | null;

  // Actions
  fetchWidgetToken: () => Promise<string>;
  fetchConnectedBanks: () => Promise<void>;
  fetchAccounts: () => Promise<void>;
  onWidgetSuccess: (event: BelvoWidgetSuccessEvent) => Promise<void>;
  disconnectBank: (linkId: string) => Promise<void>;
  syncBank: (linkId: string) => Promise<void>;
  clearError: () => void;
}

export function useBelvo(): UseBelvoReturn {
  const [connectedBanks, setConnectedBanks] = useState<ConnectedBank[]>([]);
  const [accounts, setAccounts] = useState<BelvoAccount[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isSyncing, setIsSyncing] = useState<Record<string, boolean>>({});
  const [error, setError] = useState<string | null>(null);
  const [widgetToken, setWidgetToken] = useState<string | null>(null);

  // Fetch connected banks on mount
  useEffect(() => {
    fetchConnectedBanks();
    fetchAccounts();
  }, []);

  const fetchWidgetToken = useCallback(async (): Promise<string> => {
    try {
      const response = await belvoApi.getWidgetToken();
      setWidgetToken(response.access_token);
      return response.access_token;
    } catch (err) {
      const message = getApiErrorMessage(err);
      setError(message);
      throw new Error(message);
    }
  }, []);

  const fetchConnectedBanks = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const banks = await belvoApi.listLinks();
      setConnectedBanks(banks);
    } catch (err) {
      setError(getApiErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  const fetchAccounts = useCallback(async () => {
    try {
      const accountsList = await belvoApi.listAccounts();
      setAccounts(accountsList);
    } catch (err) {
      // Don't set error for accounts - they might not exist yet
      console.error('Failed to fetch accounts:', err);
    }
  }, []);

  const onWidgetSuccess = useCallback(
    async (event: BelvoWidgetSuccessEvent) => {
      setIsLoading(true);
      setError(null);
      try {
        // Register the link in our backend
        const newBank = await belvoApi.createLink({
          link_id: event.link,
          institution: event.institution,
        });

        // Add to state
        setConnectedBanks((prev) => [newBank, ...prev]);

        // Fetch a new widget token (tokens are single-use)
        await fetchWidgetToken();

        // Refresh accounts after a short delay (sync is async)
        setTimeout(() => {
          fetchAccounts();
        }, 2000);
      } catch (err) {
        setError(getApiErrorMessage(err));
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [fetchAccounts, fetchWidgetToken]
  );

  const disconnectBank = useCallback(async (linkId: string) => {
    setIsLoading(true);
    setError(null);
    try {
      await belvoApi.deleteLink(linkId);
      setConnectedBanks((prev) => prev.filter((bank) => bank.id !== linkId));
      // Also remove accounts for this link
      setAccounts((prev) => prev.filter((acc) => acc.link_id !== linkId));
    } catch (err) {
      setError(getApiErrorMessage(err));
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const syncBank = useCallback(async (linkId: string) => {
    setIsSyncing((prev) => ({ ...prev, [linkId]: true }));
    setError(null);
    try {
      await belvoApi.triggerSync(linkId);
      // Update the bank's last_synced_at after a delay
      setTimeout(() => {
        fetchConnectedBanks();
        fetchAccounts();
      }, 3000);
    } catch (err) {
      setError(getApiErrorMessage(err));
    } finally {
      setIsSyncing((prev) => ({ ...prev, [linkId]: false }));
    }
  }, [fetchConnectedBanks, fetchAccounts]);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    connectedBanks,
    accounts,
    isLoading,
    isSyncing,
    error,
    widgetToken,
    fetchWidgetToken,
    fetchConnectedBanks,
    fetchAccounts,
    onWidgetSuccess,
    disconnectBank,
    syncBank,
    clearError,
  };
}
