/**
 * Top 10 expenses table component.
 */

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { formatCurrency, formatDate } from "@/lib/format";
import type { TopExpensesResponse } from "@/types/analytics";

interface TopExpensesProps {
  data: TopExpensesResponse | null;
  isLoading: boolean;
}

export function TopExpenses({ data, isLoading }: TopExpensesProps) {
  if (isLoading) {
    return (
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-base font-medium">Top 10 Gastos</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex h-48 items-center justify-center">
            <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
          </div>
        </CardContent>
      </Card>
    );
  }

  if (!data || data.expenses.length === 0) {
    return (
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-base font-medium">Top 10 Gastos</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex h-48 items-center justify-center text-muted-foreground">
            No hay gastos registrados
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-base font-medium">Top 10 Gastos</CardTitle>
      </CardHeader>
      <CardContent className="px-0">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="pl-6">Fecha</TableHead>
              <TableHead>Descripción</TableHead>
              <TableHead>Categoría</TableHead>
              <TableHead className="text-right pr-6">Monto</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {data.expenses.map((expense) => (
              <TableRow key={expense.id}>
                <TableCell className="pl-6 text-muted-foreground text-sm">
                  {formatDate(expense.date)}
                </TableCell>
                <TableCell className="max-w-[200px] truncate text-sm">
                  {expense.description}
                </TableCell>
                <TableCell>
                  {expense.category_name ? (
                    <span
                      className="inline-flex items-center gap-1.5 text-sm"
                      style={{ color: expense.category_color || undefined }}
                    >
                      <span
                        className="h-2 w-2 rounded-full"
                        style={{
                          backgroundColor: expense.category_color || "#6b7280",
                        }}
                      />
                      {expense.category_name}
                    </span>
                  ) : (
                    <span className="text-muted-foreground text-sm">—</span>
                  )}
                </TableCell>
                <TableCell className="text-right pr-6 font-medium text-rose-600">
                  {formatCurrency(expense.amount)}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}
