/**
 * Dashboard page - shows transactions and categories.
 */

import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Building2, FileText } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useAuth, useBanks, useCategories, useTransactions } from "@/hooks";
import {
  TransactionList,
  TransactionFilters,
  CategorySelectDialog,
} from "@/components/transactions";
import { PdfUpload } from "@/components/upload";
import type { Transaction } from "@/types";

export function DashboardPage() {
  const { t } = useTranslation();
  const { user, logout } = useAuth();
  const navigate = useNavigate();

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
    fetchTransactions,
  } = useTransactions();

  // Categories state
  const { categories, isLoading: isLoadingCategories } = useCategories();

  // Banks state
  const { banks, refetch: refetchBanks } = useBanks();

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
    <div className="min-h-screen bg-gray-100">
      <header className="bg-white shadow">
        <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-6 sm:px-6 lg:px-8">
          <h1 className="text-3xl font-bold tracking-tight text-gray-900">
            Gasticos
          </h1>
          <div className="flex items-center gap-4">
            <span className="text-sm text-gray-600">
              {t("auth.greeting", { name: user?.name })}
            </span>
            <Button variant="outline" onClick={logout}>
              {t("auth.logout")}
            </Button>
          </div>
        </div>
      </header>

      <main>
        <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8 space-y-6">
          {/* Import data card */}
          <Card>
            <CardHeader>
              <CardTitle>Importar datos</CardTitle>
            </CardHeader>
            <CardContent>
              <Tabs defaultValue="pdf" className="w-full">
                <TabsList className="grid w-full grid-cols-2">
                  <TabsTrigger value="pdf">
                    <FileText className="mr-2 h-4 w-4" />
                    Subir PDF
                  </TabsTrigger>
                  <TabsTrigger value="bank">
                    <Building2 className="mr-2 h-4 w-4" />
                    Conectar banco
                  </TabsTrigger>
                </TabsList>
                <TabsContent value="pdf" className="mt-4">
                  <PdfUpload
                    onUploadComplete={() => {
                      fetchTransactions();
                      refetchBanks();
                    }}
                  />
                </TabsContent>
                <TabsContent value="bank" className="mt-4">
                  <p className="text-gray-600">{t("connect.description")}</p>
                  <Button className="mt-4" onClick={() => navigate("/connect")}>
                    <Building2 className="mr-2 h-4 w-4" />
                    {t("belvo.connectBank")}
                  </Button>
                </TabsContent>
              </Tabs>
            </CardContent>
          </Card>

          {/* Transactions card */}
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
      </main>

      {/* Category select dialog */}
      <CategorySelectDialog
        transaction={selectedTransaction}
        categories={categories}
        isOpen={isCategoryDialogOpen}
        isUpdating={isUpdating}
        onClose={handleCloseDialog}
        onSelectCategory={handleCategorySelect}
      />
    </div>
  );
}
