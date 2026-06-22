/**
 * Top merchants spending table component.
 */

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { formatCurrency } from "@/lib/format";
import type { TopMerchantsResponse } from "@/types/analytics";

interface TopMerchantsProps {
  data: TopMerchantsResponse | null;
  isLoading: boolean;
}

export function TopMerchants({ data, isLoading }: TopMerchantsProps) {
  if (isLoading) {
    return (
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-base font-medium">Top Comercios</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex h-48 items-center justify-center">
            <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
          </div>
        </CardContent>
      </Card>
    );
  }

  if (!data || data.merchants.length === 0) {
    return (
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-base font-medium">Top Comercios</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex h-48 items-center justify-center text-muted-foreground">
            No hay comercios registrados
          </div>
        </CardContent>
      </Card>
    );
  }

  // Get max percentage for scaling progress bars
  const maxPercentage = Math.max(...data.merchants.map((m) => m.percentage));

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base font-medium">Top Comercios</CardTitle>
          <span className="text-sm text-muted-foreground">
            Total: {formatCurrency(data.total_spending)}
          </span>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {data.merchants.map((merchant, index) => (
            <div key={merchant.merchant} className="space-y-1.5">
              <div className="flex items-center justify-between text-sm">
                <div className="flex items-center gap-2">
                  <span className="text-muted-foreground w-5">{index + 1}</span>
                  <span className="font-medium truncate max-w-[180px]">
                    {merchant.merchant}
                  </span>
                </div>
                <div className="flex items-center gap-3">
                  <span className="text-muted-foreground">
                    {merchant.transaction_count}{" "}
                    {merchant.transaction_count === 1 ? "tx" : "txs"}
                  </span>
                  <span className="font-medium w-24 text-right">
                    {formatCurrency(merchant.total_amount)}
                  </span>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Progress
                  value={(merchant.percentage / maxPercentage) * 100}
                  className="h-2"
                />
                <span className="text-xs text-muted-foreground w-12 text-right">
                  {merchant.percentage.toFixed(1)}%
                </span>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
