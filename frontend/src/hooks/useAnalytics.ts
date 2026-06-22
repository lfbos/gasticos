/**
 * Analytics hook for dashboard data.
 * DOM-free for React Native reuse.
 */

import { useCallback, useEffect, useState } from "react";
import { analyticsApi } from "@/api/analytics";
import { getApiErrorMessage } from "@/api/client";
import type {
  SummaryResponse,
  SpendingByCategoryResponse,
  MonthlyTrendsResponse,
  TopExpensesResponse,
  TopMerchantsResponse,
  AnalyticsQuery,
} from "@/types/analytics";

interface UseAnalyticsReturn {
  // State
  summary: SummaryResponse | null;
  spendingByCategory: SpendingByCategoryResponse | null;
  monthlyTrends: MonthlyTrendsResponse | null;
  topExpenses: TopExpensesResponse | null;
  topMerchants: TopMerchantsResponse | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchSummary: (query?: AnalyticsQuery) => Promise<void>;
  fetchSpendingByCategory: (query?: AnalyticsQuery) => Promise<void>;
  fetchMonthlyTrends: (query?: AnalyticsQuery) => Promise<void>;
  fetchTopExpenses: (query?: AnalyticsQuery) => Promise<void>;
  fetchTopMerchants: (query?: AnalyticsQuery) => Promise<void>;
  fetchAll: (query?: AnalyticsQuery) => Promise<void>;
  clearError: () => void;
}

export function useAnalytics(): UseAnalyticsReturn {
  const [summary, setSummary] = useState<SummaryResponse | null>(null);
  const [spendingByCategory, setSpendingByCategory] =
    useState<SpendingByCategoryResponse | null>(null);
  const [monthlyTrends, setMonthlyTrends] =
    useState<MonthlyTrendsResponse | null>(null);
  const [topExpenses, setTopExpenses] = useState<TopExpensesResponse | null>(
    null,
  );
  const [topMerchants, setTopMerchants] = useState<TopMerchantsResponse | null>(
    null,
  );
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchSummary = useCallback(async (query?: AnalyticsQuery) => {
    try {
      const data = await analyticsApi.getSummary(query);
      setSummary(data);
    } catch (err) {
      const message = getApiErrorMessage(err);
      setError(message);
    }
  }, []);

  const fetchSpendingByCategory = useCallback(
    async (query?: AnalyticsQuery) => {
      try {
        const data = await analyticsApi.getSpendingByCategory(query);
        setSpendingByCategory(data);
      } catch (err) {
        const message = getApiErrorMessage(err);
        setError(message);
      }
    },
    [],
  );

  const fetchMonthlyTrends = useCallback(async (query?: AnalyticsQuery) => {
    try {
      const data = await analyticsApi.getMonthlyTrends(query);
      setMonthlyTrends(data);
    } catch (err) {
      const message = getApiErrorMessage(err);
      setError(message);
    }
  }, []);

  const fetchTopExpenses = useCallback(async (query?: AnalyticsQuery) => {
    try {
      const data = await analyticsApi.getTopExpenses(query);
      setTopExpenses(data);
    } catch (err) {
      const message = getApiErrorMessage(err);
      setError(message);
    }
  }, []);

  const fetchTopMerchants = useCallback(async (query?: AnalyticsQuery) => {
    try {
      const data = await analyticsApi.getTopMerchants(query);
      setTopMerchants(data);
    } catch (err) {
      const message = getApiErrorMessage(err);
      setError(message);
    }
  }, []);

  const fetchAll = useCallback(
    async (query?: AnalyticsQuery) => {
      setIsLoading(true);
      setError(null);
      try {
        await Promise.all([
          fetchSummary(query),
          fetchSpendingByCategory(query),
          fetchMonthlyTrends(query),
          fetchTopExpenses(query),
          fetchTopMerchants(query),
        ]);
      } finally {
        setIsLoading(false);
      }
    },
    [
      fetchSummary,
      fetchSpendingByCategory,
      fetchMonthlyTrends,
      fetchTopExpenses,
      fetchTopMerchants,
    ],
  );

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  // Fetch all analytics on mount
  useEffect(() => {
    fetchAll();
  }, [fetchAll]);

  return {
    summary,
    spendingByCategory,
    monthlyTrends,
    topExpenses,
    topMerchants,
    isLoading,
    error,
    fetchSummary,
    fetchSpendingByCategory,
    fetchMonthlyTrends,
    fetchTopExpenses,
    fetchTopMerchants,
    fetchAll,
    clearError,
  };
}
