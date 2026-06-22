/**
 * Analytics types for dashboard charts and summaries.
 */

/**
 * Financial summary response.
 */
export interface SummaryResponse {
  total_income: number;
  total_expenses: number;
  net_balance: number;
  transaction_count: number;
  days_in_period: number;
  daily_average: number;
  savings_rate: number;
  largest_expense: number;
}

/**
 * Category breakdown for spending chart.
 */
export interface CategoryBreakdown {
  category_id: string | null;
  category_name: string;
  category_key: string | null;
  category_color: string | null;
  total_amount: number;
  transaction_count: number;
  percentage: number;
}

/**
 * Spending by category response.
 */
export interface SpendingByCategoryResponse {
  categories: CategoryBreakdown[];
  total_spending: number;
}

/**
 * Monthly data point for trends chart.
 */
export interface MonthlyDataPoint {
  year: number;
  month: number;
  month_name: string;
  income: number;
  expenses: number;
  net: number;
}

/**
 * Monthly trends response.
 */
export interface MonthlyTrendsResponse {
  data: MonthlyDataPoint[];
}

/**
 * Query parameters for analytics endpoints.
 */
export interface AnalyticsQuery {
  start_date?: string;
  end_date?: string;
  months?: number;
  category_ids?: string[];
  is_income?: boolean;
}

/**
 * Top expense item.
 */
export interface TopExpenseItem {
  id: string;
  date: string;
  description: string;
  amount: number;
  category_name: string | null;
  category_color: string | null;
}

/**
 * Top expenses response.
 */
export interface TopExpensesResponse {
  expenses: TopExpenseItem[];
}

/**
 * Merchant spending aggregate.
 */
export interface MerchantSpending {
  merchant: string;
  total_amount: number;
  transaction_count: number;
  percentage: number;
}

/**
 * Top merchants response.
 */
export interface TopMerchantsResponse {
  merchants: MerchantSpending[];
  total_spending: number;
}
