/**
 * Analytics API client.
 */

import { apiClient } from "./client";
import type {
  SummaryResponse,
  SpendingByCategoryResponse,
  MonthlyTrendsResponse,
  TopExpensesResponse,
  TopMerchantsResponse,
  AnalyticsQuery,
} from "@/types/analytics";

/**
 * Convert AnalyticsQuery to API params (serializing arrays).
 */
function toApiParams(query?: AnalyticsQuery): Record<string, unknown> {
  if (!query) return {};

  return {
    ...query,
    // Serialize category_ids array as comma-separated string
    category_ids: query.category_ids?.join(","),
  };
}

/**
 * Get financial summary (income, expenses, balance).
 */
async function getSummary(query?: AnalyticsQuery): Promise<SummaryResponse> {
  const response = await apiClient.get<SummaryResponse>("/analytics/summary", {
    params: toApiParams(query),
  });
  return response.data;
}

/**
 * Get spending breakdown by category.
 */
async function getSpendingByCategory(
  query?: AnalyticsQuery,
): Promise<SpendingByCategoryResponse> {
  const response = await apiClient.get<SpendingByCategoryResponse>(
    "/analytics/by-category",
    { params: toApiParams(query) },
  );
  return response.data;
}

/**
 * Get monthly income/expense trends.
 */
async function getMonthlyTrends(
  query?: AnalyticsQuery,
): Promise<MonthlyTrendsResponse> {
  const response = await apiClient.get<MonthlyTrendsResponse>(
    "/analytics/monthly",
    { params: toApiParams(query) },
  );
  return response.data;
}

/**
 * Get top expense transactions.
 */
async function getTopExpenses(
  query?: AnalyticsQuery,
): Promise<TopExpensesResponse> {
  const response = await apiClient.get<TopExpensesResponse>(
    "/analytics/top-expenses",
    { params: toApiParams(query) },
  );
  return response.data;
}

/**
 * Get top spending merchants.
 */
async function getTopMerchants(
  query?: AnalyticsQuery,
): Promise<TopMerchantsResponse> {
  const response = await apiClient.get<TopMerchantsResponse>(
    "/analytics/top-merchants",
    { params: toApiParams(query) },
  );
  return response.data;
}

export const analyticsApi = {
  getSummary,
  getSpendingByCategory,
  getMonthlyTrends,
  getTopExpenses,
  getTopMerchants,
};
