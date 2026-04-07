/**
 * CategorySelectDialog component for changing a transaction's category.
 */

import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { CategoryList } from "@/components/categories";
import type { Transaction, Category } from "@/types";
import {
  formatTransactionAmount,
  formatTransactionDate,
} from "@/types/transaction";

interface CategorySelectDialogProps {
  transaction: Transaction | null;
  categories: Category[];
  isOpen: boolean;
  isUpdating?: boolean;
  onClose: () => void;
  onSelectCategory: (transactionId: string, categoryId: string | null) => void;
}

export function CategorySelectDialog({
  transaction,
  categories,
  isOpen,
  isUpdating = false,
  onClose,
  onSelectCategory,
}: CategorySelectDialogProps) {
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(
    transaction?.category_id ?? null,
  );

  // Reset selection when transaction changes
  if (
    transaction &&
    selectedCategoryId !== transaction.category_id &&
    !isUpdating
  ) {
    setSelectedCategoryId(transaction.category_id);
  }

  const handleCategorySelect = (category: Category | null) => {
    setSelectedCategoryId(category?.id ?? null);
  };

  const handleSave = () => {
    if (transaction) {
      onSelectCategory(transaction.id, selectedCategoryId);
    }
  };

  const handleRemoveCategory = () => {
    if (transaction) {
      onSelectCategory(transaction.id, null);
    }
  };

  if (!transaction) return null;

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Categorizar transacción</DialogTitle>
          <DialogDescription>
            Selecciona una categoría para esta transacción
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Transaction preview */}
          <div className="bg-gray-50 rounded-lg p-3 space-y-1">
            <p className="text-sm text-gray-500">
              {formatTransactionDate(transaction.date)}
            </p>
            <p className="font-medium truncate" title={transaction.description}>
              {transaction.description}
            </p>
            <p
              className={`text-lg font-semibold ${
                transaction.is_income ? "text-green-600" : "text-red-600"
              }`}
            >
              {transaction.is_income ? "+" : "-"}
              {formatTransactionAmount(transaction.amount)}
            </p>
          </div>

          {/* Category selection */}
          <CategoryList
            categories={categories}
            selectedCategoryId={selectedCategoryId}
            onSelectCategory={handleCategorySelect}
            showSystemLabel
          />
        </div>

        <DialogFooter className="gap-2 sm:gap-0">
          {transaction.category_id && (
            <Button
              variant="outline"
              onClick={handleRemoveCategory}
              disabled={isUpdating}
              className="text-red-600 hover:text-red-700"
            >
              Quitar categoría
            </Button>
          )}
          <div className="flex gap-2 ml-auto">
            <Button variant="outline" onClick={onClose} disabled={isUpdating}>
              Cancelar
            </Button>
            <Button
              onClick={handleSave}
              disabled={
                isUpdating || selectedCategoryId === transaction.category_id
              }
            >
              {isUpdating ? "Guardando..." : "Guardar"}
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
