/**
 * Dashboard page - shows analytics with filters.
 */

import { useState, useEffect } from "react";
import { AppLayout } from "@/components/layout";
import { useAnalytics, useCategories } from "@/hooks";
import {
  AnalyticsFilters,
  SummaryCards,
  SpendingByCategory,
  MonthlyTrends,
  TopExpenses,
  TopMerchants,
} from "@/components/analytics";
import type { AnalyticsQuery } from "@/types/analytics";

export function DashboardPage() {
  const [filters, setFilters] = useState<AnalyticsQuery>({});

  const {
    summary,
    spendingByCategory,
    monthlyTrends,
    topExpenses,
    topMerchants,
    isLoading,
    fetchAll,
  } = useAnalytics();
  const { categories } = useCategories();

  // Refetch analytics when filters change
  useEffect(() => {
    fetchAll(filters);
  }, [filters, fetchAll]);

  return (
    <AppLayout>
      <div className="space-y-6">
        {/* Header with filters */}
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <h2 className="text-lg font-semibold">Resumen</h2>
          <AnalyticsFilters
            filters={filters}
            onFiltersChange={setFilters}
            categories={categories}
          />
        </div>

        {/* Summary cards */}
        <SummaryCards data={summary} isLoading={isLoading} />

        {/* Charts row */}
        <div className="grid gap-6 lg:grid-cols-2">
          <SpendingByCategory data={spendingByCategory} isLoading={isLoading} />
          <MonthlyTrends data={monthlyTrends} isLoading={isLoading} />
        </div>

        {/* Tables row */}
        <div className="grid gap-6 lg:grid-cols-2">
          <TopExpenses data={topExpenses} isLoading={isLoading} />
          <TopMerchants data={topMerchants} isLoading={isLoading} />
        </div>
      </div>
    </AppLayout>
  );
}
