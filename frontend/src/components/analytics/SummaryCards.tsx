/**
 * Summary cards showing income, expenses, balance, and additional stats.
 */

import {
  TrendingUp,
  TrendingDown,
  Wallet,
  PiggyBank,
  CalendarDays,
  Zap,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatCurrency } from "@/lib/format";
import type { SummaryResponse } from "@/types/analytics";
import { cn } from "@/lib/utils";

interface SummaryCardsProps {
  data: SummaryResponse | null;
  isLoading: boolean;
}

interface SummaryCardProps {
  title: string;
  value: string;
  icon: React.ReactNode;
  subtitle?: string;
  valueClassName?: string;
}

function SummaryCard({
  title,
  value,
  icon,
  subtitle,
  valueClassName,
}: SummaryCardProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        {icon}
      </CardHeader>
      <CardContent>
        <div className={cn("text-2xl font-bold", valueClassName)}>{value}</div>
        {subtitle && (
          <p className="text-xs text-muted-foreground mt-1">{subtitle}</p>
        )}
      </CardContent>
    </Card>
  );
}

function LoadingCard() {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <div className="h-4 w-24 animate-pulse rounded bg-muted" />
        <div className="h-4 w-4 animate-pulse rounded bg-muted" />
      </CardHeader>
      <CardContent>
        <div className="h-8 w-32 animate-pulse rounded bg-muted" />
      </CardContent>
    </Card>
  );
}

export function SummaryCards({ data, isLoading }: SummaryCardsProps) {
  if (isLoading) {
    return (
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
        <LoadingCard />
        <LoadingCard />
        <LoadingCard />
        <LoadingCard />
        <LoadingCard />
        <LoadingCard />
      </div>
    );
  }

  const summary = data ?? {
    total_income: 0,
    total_expenses: 0,
    net_balance: 0,
    transaction_count: 0,
    days_in_period: 1,
    daily_average: 0,
    savings_rate: 0,
    largest_expense: 0,
  };

  const savingsRateColor =
    summary.savings_rate >= 20
      ? "text-emerald-600"
      : summary.savings_rate >= 0
        ? "text-amber-600"
        : "text-rose-600";

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
      <SummaryCard
        title="Ingresos"
        value={formatCurrency(summary.total_income)}
        icon={<TrendingUp className="h-4 w-4 text-emerald-500" />}
        valueClassName="text-emerald-600"
      />
      <SummaryCard
        title="Gastos"
        value={formatCurrency(summary.total_expenses)}
        icon={<TrendingDown className="h-4 w-4 text-rose-500" />}
        subtitle={`${summary.transaction_count} transacciones`}
        valueClassName="text-rose-600"
      />
      <SummaryCard
        title="Balance"
        value={formatCurrency(summary.net_balance)}
        icon={<Wallet className="h-4 w-4 text-blue-500" />}
        valueClassName={
          summary.net_balance >= 0 ? "text-emerald-600" : "text-rose-600"
        }
      />
      <SummaryCard
        title="Tasa de Ahorro"
        value={`${summary.savings_rate.toFixed(1)}%`}
        icon={<PiggyBank className="h-4 w-4 text-violet-500" />}
        subtitle={summary.savings_rate >= 20 ? "Excelente" : "Mejorable"}
        valueClassName={savingsRateColor}
      />
      <SummaryCard
        title="Promedio Diario"
        value={formatCurrency(summary.daily_average)}
        icon={<CalendarDays className="h-4 w-4 text-amber-500" />}
        subtitle={`${summary.days_in_period} días`}
      />
      <SummaryCard
        title="Mayor Gasto"
        value={formatCurrency(summary.largest_expense)}
        icon={<Zap className="h-4 w-4 text-rose-500" />}
        valueClassName="text-rose-600"
      />
    </div>
  );
}
