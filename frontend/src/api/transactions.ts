/**
 * Transactions API client.
 */

import { apiClient } from "./client";
import type {
  Transaction,
  PaginatedTransactionsResponse,
  ListTransactionsParams,
  UpdateTransactionCategoryRequest,
} from "@/types/transaction";

/**
 * Build query string from params.
 */
function buildQueryString(params: ListTransactionsParams): string {
  const searchParams = new URLSearchParams();

  if (params.page !== undefined) {
    searchParams.set("page", params.page.toString());
  }
  if (params.per_page !== undefined) {
    searchParams.set("per_page", params.per_page.toString());
  }
  if (params.category_id) {
    searchParams.set("category_id", params.category_id);
  }
  if (params.date_from) {
    searchParams.set("date_from", params.date_from);
  }
  if (params.date_to) {
    searchParams.set("date_to", params.date_to);
  }
  if (params.is_income !== undefined) {
    searchParams.set("is_income", params.is_income.toString());
  }
  if (params.search) {
    searchParams.set("search", params.search);
  }
  if (params.uncategorized !== undefined) {
    searchParams.set("uncategorized", params.uncategorized.toString());
  }

  const queryString = searchParams.toString();
  return queryString ? `?${queryString}` : "";
}

/**
 * Get paginated transactions with optional filters.
 */
async function getTransactions(
  params: ListTransactionsParams = {},
): Promise<PaginatedTransactionsResponse> {
  const queryString = buildQueryString(params);
  const response = await apiClient.get<PaginatedTransactionsResponse>(
    `/transactions${queryString}`,
  );
  return response.data;
}

/**
 * Get a single transaction by ID.
 */
async function getTransaction(id: string): Promise<Transaction> {
  const response = await apiClient.get<Transaction>(`/transactions/${id}`);
  return response.data;
}

/**
 * Update a transaction's category.
 */
async function updateTransactionCategory(
  id: string,
  data: UpdateTransactionCategoryRequest,
): Promise<Transaction> {
  const response = await apiClient.patch<Transaction>(
    `/transactions/${id}`,
    data,
  );
  return response.data;
}

export const transactionsApi = {
  getTransactions,
  getTransaction,
  updateTransactionCategory,
};
