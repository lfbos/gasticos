/**
 * TransactionList component for displaying paginated transactions.
 */

import {
  Table,
  TableBody,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";
import { Button } from "@/components/ui/button";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { TransactionRow } from "./TransactionRow";
import type { Transaction, PaginationMeta } from "@/types";

interface TransactionListProps {
  transactions: Transaction[];
  meta: PaginationMeta | null;
  isLoading?: boolean;
  onCategoryClick?: (transaction: Transaction) => void;
  onPageChange?: (page: number) => void;
  className?: string;
}

function TransactionListSkeleton() {
  return (
    <div className="space-y-3">
      {Array.from({ length: 5 }).map((_, i) => (
        <div key={i} className="flex items-center gap-4">
          <Skeleton className="h-4 w-20" />
          <Skeleton className="h-4 flex-1" />
          <Skeleton className="h-6 w-24 rounded-full" />
          <Skeleton className="h-4 w-24" />
        </div>
      ))}
    </div>
  );
}

function EmptyState() {
  return (
    <div className="text-center py-12">
      <p className="text-gray-500">No hay transacciones para mostrar</p>
      <p className="text-sm text-gray-400 mt-1">
        Las transacciones aparecerán aquí después de conectar tu banco
      </p>
    </div>
  );
}

function Pagination({
  meta,
  onPageChange,
}: {
  meta: PaginationMeta;
  onPageChange: (page: number) => void;
}) {
  const { page, total_pages, total } = meta;
  const hasPrevious = page > 1;
  const hasNext = page < total_pages;

  return (
    <div className="flex items-center justify-between px-2 py-4">
      <p className="text-sm text-gray-500">
        {total} {total === 1 ? "transacción" : "transacciones"} en total
      </p>
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={() => onPageChange(page - 1)}
          disabled={!hasPrevious}
        >
          <ChevronLeft className="h-4 w-4" />
          Anterior
        </Button>
        <span className="text-sm text-gray-600">
          Página {page} de {total_pages}
        </span>
        <Button
          variant="outline"
          size="sm"
          onClick={() => onPageChange(page + 1)}
          disabled={!hasNext}
        >
          Siguiente
          <ChevronRight className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}

export function TransactionList({
  transactions,
  meta,
  isLoading = false,
  onCategoryClick,
  onPageChange,
  className = "",
}: TransactionListProps) {
  if (isLoading) {
    return (
      <div className={className}>
        <TransactionListSkeleton />
      </div>
    );
  }

  if (transactions.length === 0) {
    return (
      <div className={className}>
        <EmptyState />
      </div>
    );
  }

  return (
    <div className={className}>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[100px]">Fecha</TableHead>
            <TableHead>Descripción</TableHead>
            <TableHead className="w-[140px]">Categoría</TableHead>
            <TableHead className="text-right w-[120px]">Monto</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {transactions.map((transaction) => (
            <TransactionRow
              key={transaction.id}
              transaction={transaction}
              onCategoryClick={onCategoryClick}
            />
          ))}
        </TableBody>
      </Table>

      {meta && meta.total_pages > 1 && onPageChange && (
        <Pagination meta={meta} onPageChange={onPageChange} />
      )}
    </div>
  );
}
