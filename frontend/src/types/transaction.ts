/**
 * Transaction types matching backend API responses.
 */

import type { PaginationMeta } from "./api";

/**
 * Transaction from the API.
 */
export interface Transaction {
  id: string;
  date: string;
  description: string;
  amount: number;
  balance: number | null;
  reference: string | null;
  is_income: boolean;
  category_id: string | null;
  category_name: string | null;
  category_key: string | null;
  category_color: string | null;
  category_icon: string | null;
  is_user_categorized: boolean;
  created_at: string;
}

/**
 * Paginated transactions response.
 */
export interface PaginatedTransactionsResponse {
  data: Transaction[];
  meta: PaginationMeta;
}

/**
 * Query parameters for listing transactions.
 */
export interface ListTransactionsParams {
  page?: number;
  per_page?: number;
  category_id?: string;
  date_from?: string;
  date_to?: string;
  is_income?: boolean;
  search?: string;
  uncategorized?: boolean;
}

/**
 * Request to update a transaction's category.
 */
export interface UpdateTransactionCategoryRequest {
  category_id: string | null;
}

/**
 * Format a transaction amount for display in Colombian format.
 */
export function formatTransactionAmount(amount: number): string {
  return new Intl.NumberFormat("es-CO", {
    style: "currency",
    currency: "COP",
    minimumFractionDigits: 0,
  }).format(Math.abs(amount));
}

/**
 * Format a transaction date for display.
 */
export function formatTransactionDate(dateStr: string): string {
  const date = new Date(dateStr);
  return new Intl.DateTimeFormat("es-CO", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  }).format(date);
}

/**
 * Get the sign prefix for a transaction amount.
 */
export function getAmountSign(isIncome: boolean): string {
  return isIncome ? "+" : "-";
}

/**
 * Get the CSS class for amount coloring.
 */
export function getAmountColorClass(isIncome: boolean): string {
  return isIncome ? "text-green-600" : "text-red-600";
}
