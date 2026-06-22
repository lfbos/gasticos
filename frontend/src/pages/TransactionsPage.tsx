/**
 * Transactions page - shows transaction list with filters and category editing.
 */

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { AppLayout } from "@/components/layout";
import { useBanks, useCategories, useTransactions } from "@/hooks";
import {
  TransactionList,
  TransactionFilters,
  CategorySelectDialog,
} from "@/components/transactions";
import type { Transaction } from "@/types";

export function TransactionsPage() {
  // Transaction state
  const {
    transactions,
    meta,
    filters,
    isLoading: isLoadingTransactions,
    isUpdating,
    setFilters,
    goToPage,
    updateTransactionCategory,
  } = useTransactions();

  // Categories state
  const { categories, isLoading: isLoadingCategories } = useCategories();

  // Banks state
  const { banks } = useBanks();

  // Category edit dialog state
  const [selectedTransaction, setSelectedTransaction] =
    useState<Transaction | null>(null);
  const [isCategoryDialogOpen, setIsCategoryDialogOpen] = useState(false);

  const handleCategoryClick = (transaction: Transaction) => {
    setSelectedTransaction(transaction);
    setIsCategoryDialogOpen(true);
  };

  const handleCategorySelect = async (
    transactionId: string,
    categoryId: string | null,
  ) => {
    try {
      await updateTransactionCategory(transactionId, categoryId);
      setIsCategoryDialogOpen(false);
      setSelectedTransaction(null);
    } catch {
      // Error is handled by hook
    }
  };

  const handleCloseDialog = () => {
    setIsCategoryDialogOpen(false);
    setSelectedTransaction(null);
  };

  const isLoading = isLoadingTransactions || isLoadingCategories;

  return (
    <AppLayout>
      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle>Transacciones</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Filters */}
            <TransactionFilters
              categories={categories}
              banks={banks}
              filters={filters}
              onFiltersChange={setFilters}
            />

            {/* Transaction list */}
            <TransactionList
              transactions={transactions}
              meta={meta}
              isLoading={isLoading}
              onCategoryClick={handleCategoryClick}
              onPageChange={goToPage}
              sortOrder={filters.sort_order ?? "desc"}
              onSortChange={(order) => setFilters({ sort_order: order })}
            />
          </CardContent>
        </Card>
      </div>

      {/* Category select dialog */}
      <CategorySelectDialog
        transaction={selectedTransaction}
        categories={categories}
        isOpen={isCategoryDialogOpen}
        isUpdating={isUpdating}
        onClose={handleCloseDialog}
        onSelectCategory={handleCategorySelect}
      />
    </AppLayout>
  );
}
