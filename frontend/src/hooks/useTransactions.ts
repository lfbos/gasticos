/**
 * Transactions hook.
 * DOM-free for React Native reuse.
 */

import { useCallback, useEffect, useState } from "react";
import { transactionsApi } from "@/api/transactions";
import { getApiErrorMessage } from "@/api/client";
import type { PaginationMeta } from "@/types/api";
import type { Transaction, ListTransactionsParams } from "@/types/transaction";

// ============================================================================
// useBanks Hook
// ============================================================================

interface UseBanksReturn {
  banks: string[];
  isLoading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
}

export function useBanks(): UseBanksReturn {
  const [banks, setBanks] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchBanks = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await transactionsApi.getBanks();
      setBanks(response.banks);
    } catch (err) {
      const message = getApiErrorMessage(err);
      setError(message);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBanks();
  }, [fetchBanks]);

  return {
    banks,
    isLoading,
    error,
    refetch: fetchBanks,
  };
}

// ============================================================================
// useTransactions Hook
// ============================================================================

interface UseTransactionsReturn {
  // State
  transactions: Transaction[];
  meta: PaginationMeta | null;
  isLoading: boolean;
  isUpdating: boolean;
  error: string | null;

  // Current filters
  filters: ListTransactionsParams;

  // Actions
  fetchTransactions: (params?: ListTransactionsParams) => Promise<void>;
  setFilters: (params: ListTransactionsParams) => void;
  updateTransactionCategory: (
    id: string,
    categoryId: string | null,
  ) => Promise<Transaction>;
  getTransactionById: (id: string) => Transaction | undefined;
  goToPage: (page: number) => Promise<void>;
  clearError: () => void;
}

const DEFAULT_FILTERS: ListTransactionsParams = {
  page: 1,
  per_page: 20,
};

export function useTransactions(
  initialFilters: ListTransactionsParams = {},
): UseTransactionsReturn {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [meta, setMeta] = useState<PaginationMeta | null>(null);
  const [filters, setFiltersState] = useState<ListTransactionsParams>({
    ...DEFAULT_FILTERS,
    ...initialFilters,
  });
  const [isLoading, setIsLoading] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchTransactions = useCallback(
    async (params?: ListTransactionsParams) => {
      const queryParams = params ?? filters;
      setIsLoading(true);
      setError(null);
      try {
        const response = await transactionsApi.getTransactions(queryParams);
        setTransactions(response.data);
        setMeta(response.meta);
      } catch (err) {
        const message = getApiErrorMessage(err);
        setError(message);
      } finally {
        setIsLoading(false);
      }
    },
    [filters],
  );

  const setFilters = useCallback((params: ListTransactionsParams) => {
    setFiltersState((prev) => ({
      ...prev,
      ...params,
      page: params.page ?? 1, // Reset to page 1 when filters change
    }));
  }, []);

  const updateTransactionCategory = useCallback(
    async (id: string, categoryId: string | null): Promise<Transaction> => {
      setIsUpdating(true);
      setError(null);
      try {
        const updated = await transactionsApi.updateTransactionCategory(id, {
          category_id: categoryId,
        });
        setTransactions((prev) => prev.map((t) => (t.id === id ? updated : t)));
        return updated;
      } catch (err) {
        const message = getApiErrorMessage(err);
        setError(message);
        throw new Error(message);
      } finally {
        setIsUpdating(false);
      }
    },
    [],
  );

  const getTransactionById = useCallback(
    (id: string): Transaction | undefined => {
      return transactions.find((t) => t.id === id);
    },
    [transactions],
  );

  const goToPage = useCallback(
    async (page: number) => {
      const newFilters = { ...filters, page };
      setFiltersState(newFilters);
      await fetchTransactions(newFilters);
    },
    [filters, fetchTransactions],
  );

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  // Fetch transactions when filters change
  useEffect(() => {
    fetchTransactions();
  }, [fetchTransactions]);

  return {
    transactions,
    meta,
    isLoading,
    isUpdating,
    error,
    filters,
    fetchTransactions,
    setFilters,
    updateTransactionCategory,
    getTransactionById,
    goToPage,
    clearError,
  };
}
