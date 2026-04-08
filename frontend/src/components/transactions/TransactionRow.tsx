/**
 * TransactionRow component for displaying a single transaction in the list.
 */

import { TableRow, TableCell } from "@/components/ui/table";
import { CategoryBadge } from "@/components/categories";
import type { Transaction, Category } from "@/types";
import {
  formatTransactionAmount,
  formatTransactionDate,
  getAmountSign,
  getAmountColorClass,
} from "@/types/transaction";

interface TransactionRowProps {
  transaction: Transaction;
  onCategoryClick?: (transaction: Transaction) => void;
}

export function TransactionRow({
  transaction,
  onCategoryClick,
}: TransactionRowProps) {
  const amountColorClass = getAmountColorClass(transaction.is_income);
  const amountSign = getAmountSign(transaction.is_income);
  const formattedAmount = formatTransactionAmount(transaction.amount);
  const formattedDate = formatTransactionDate(transaction.date);

  // Build a category object from transaction data for CategoryBadge
  const category: Category | null =
    transaction.category_id && transaction.category_name
      ? {
          id: transaction.category_id,
          name: transaction.category_name,
          key: transaction.category_key,
          icon: transaction.category_icon,
          color: transaction.category_color,
          is_system: false, // Not needed for display
        }
      : null;

  return (
    <TableRow>
      <TableCell className="font-medium whitespace-nowrap">
        {formattedDate}
      </TableCell>
      <TableCell className="truncate max-w-0" title={transaction.description}>
        {transaction.description}
      </TableCell>
      <TableCell className="text-sm text-gray-600 whitespace-nowrap">
        {transaction.bank ?? "-"}
      </TableCell>
      <TableCell>
        {category ? (
          <button
            type="button"
            onClick={() => onCategoryClick?.(transaction)}
            className="hover:opacity-80 transition-opacity cursor-pointer"
            title="Cambiar categoría"
          >
            <CategoryBadge category={category} size="sm" />
          </button>
        ) : (
          <button
            type="button"
            onClick={() => onCategoryClick?.(transaction)}
            className="text-xs text-gray-400 hover:text-gray-600 transition-colors cursor-pointer px-2 py-1 border border-dashed border-gray-300 rounded-full hover:border-gray-400"
          >
            Sin categoría
          </button>
        )}
      </TableCell>
      <TableCell className={`text-right font-medium ${amountColorClass}`}>
        {amountSign}
        {formattedAmount}
      </TableCell>
    </TableRow>
  );
}
