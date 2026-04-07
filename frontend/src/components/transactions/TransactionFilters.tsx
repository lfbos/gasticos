/**
 * TransactionFilters component for filtering transactions.
 */

import { useState, useEffect } from "react";
import { Search, X } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import type { Category, ListTransactionsParams } from "@/types";
import { getCategoryDisplayName } from "@/types/category";

interface TransactionFiltersProps {
  categories: Category[];
  filters: ListTransactionsParams;
  onFiltersChange: (filters: ListTransactionsParams) => void;
  className?: string;
}

export function TransactionFilters({
  categories,
  filters,
  onFiltersChange,
  className = "",
}: TransactionFiltersProps) {
  // Local state for controlled inputs
  const [search, setSearch] = useState(filters.search ?? "");
  const [dateFrom, setDateFrom] = useState(filters.date_from ?? "");
  const [dateTo, setDateTo] = useState(filters.date_to ?? "");

  // Debounce search input
  useEffect(() => {
    const timer = setTimeout(() => {
      if (search !== (filters.search ?? "")) {
        onFiltersChange({ ...filters, search: search || undefined });
      }
    }, 300);
    return () => clearTimeout(timer);
  }, [search, filters, onFiltersChange]);

  const handleCategoryChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    onFiltersChange({
      ...filters,
      category_id: value || undefined,
    });
  };

  const handleTypeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    onFiltersChange({
      ...filters,
      is_income: value === "" ? undefined : value === "income",
    });
  };

  const handleDateFromChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setDateFrom(value);
    onFiltersChange({
      ...filters,
      date_from: value || undefined,
    });
  };

  const handleDateToChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setDateTo(value);
    onFiltersChange({
      ...filters,
      date_to: value || undefined,
    });
  };

  const handleUncategorizedChange = (
    e: React.ChangeEvent<HTMLInputElement>
  ) => {
    onFiltersChange({
      ...filters,
      uncategorized: e.target.checked || undefined,
    });
  };

  const clearFilters = () => {
    setSearch("");
    setDateFrom("");
    setDateTo("");
    onFiltersChange({
      page: 1,
      per_page: filters.per_page,
    });
  };

  const hasActiveFilters =
    filters.search ||
    filters.category_id ||
    filters.date_from ||
    filters.date_to ||
    filters.is_income !== undefined ||
    filters.uncategorized;

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Search and type row */}
      <div className="flex flex-col sm:flex-row gap-3">
        {/* Search */}
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
          <Input
            type="text"
            placeholder="Buscar transacciones..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
          />
        </div>

        {/* Type filter */}
        <select
          value={
            filters.is_income === undefined
              ? ""
              : filters.is_income
                ? "income"
                : "expense"
          }
          onChange={handleTypeChange}
          className="h-9 rounded-md border border-input bg-background px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
        >
          <option value="">Todos los tipos</option>
          <option value="income">Ingresos</option>
          <option value="expense">Gastos</option>
        </select>

        {/* Category filter */}
        <select
          value={filters.category_id ?? ""}
          onChange={handleCategoryChange}
          className="h-9 rounded-md border border-input bg-background px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
        >
          <option value="">Todas las categorías</option>
          {categories.map((cat) => (
            <option key={cat.id} value={cat.id}>
              {getCategoryDisplayName(cat)}
            </option>
          ))}
        </select>
      </div>

      {/* Date range and uncategorized row */}
      <div className="flex flex-col sm:flex-row gap-3 items-end">
        {/* Date from */}
        <div className="flex-1 space-y-1">
          <Label htmlFor="date-from" className="text-xs text-gray-500">
            Desde
          </Label>
          <Input
            id="date-from"
            type="date"
            value={dateFrom}
            onChange={handleDateFromChange}
          />
        </div>

        {/* Date to */}
        <div className="flex-1 space-y-1">
          <Label htmlFor="date-to" className="text-xs text-gray-500">
            Hasta
          </Label>
          <Input
            id="date-to"
            type="date"
            value={dateTo}
            onChange={handleDateToChange}
          />
        </div>

        {/* Uncategorized checkbox */}
        <label className="flex items-center gap-2 h-9 px-3 cursor-pointer">
          <input
            type="checkbox"
            checked={filters.uncategorized ?? false}
            onChange={handleUncategorizedChange}
            className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
          />
          <span className="text-sm whitespace-nowrap">Sin categoría</span>
        </label>

        {/* Clear filters */}
        {hasActiveFilters && (
          <Button
            variant="ghost"
            size="sm"
            onClick={clearFilters}
            className="text-gray-500"
          >
            <X className="h-4 w-4 mr-1" />
            Limpiar
          </Button>
        )}
      </div>
    </div>
  );
}
