/**
 * TransactionFilters component for filtering transactions.
 */

import { useState, useEffect, useRef } from "react";
import {
  Search,
  X,
  ChevronDown,
  Check,
  Home,
  ShoppingCart,
  Utensils,
  Car,
  HeartPulse,
  GraduationCap,
  Gamepad,
  Shirt,
  Landmark,
  Laptop,
  Repeat,
  CircleDot,
  Wallet,
  Zap,
  ArrowRightLeft,
  type LucideIcon,
} from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import type { Category, ListTransactionsParams } from "@/types";
import { getCategoryDisplayName, getCategoryColor } from "@/types/category";

const ICON_MAP: Record<string, LucideIcon> = {
  home: Home,
  "shopping-cart": ShoppingCart,
  utensils: Utensils,
  car: Car,
  "heart-pulse": HeartPulse,
  "graduation-cap": GraduationCap,
  gamepad: Gamepad,
  shirt: Shirt,
  landmark: Landmark,
  laptop: Laptop,
  repeat: Repeat,
  "circle-dot": CircleDot,
  wallet: Wallet,
  zap: Zap,
  "arrow-right-left": ArrowRightLeft,
};

interface TransactionFiltersProps {
  categories: Category[];
  banks: string[];
  filters: ListTransactionsParams;
  onFiltersChange: (filters: ListTransactionsParams) => void;
  className?: string;
}

export function TransactionFilters({
  categories,
  banks,
  filters,
  onFiltersChange,
  className = "",
}: TransactionFiltersProps) {
  // Local state for controlled inputs
  const [search, setSearch] = useState(filters.search ?? "");
  const [dateFrom, setDateFrom] = useState(filters.date_from ?? "");
  const [dateTo, setDateTo] = useState(filters.date_to ?? "");
  const [categoryDropdownOpen, setCategoryDropdownOpen] = useState(false);
  const categoryDropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        categoryDropdownRef.current &&
        !categoryDropdownRef.current.contains(event.target as Node)
      ) {
        setCategoryDropdownOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  // Debounce search input
  useEffect(() => {
    const timer = setTimeout(() => {
      if (search !== (filters.search ?? "")) {
        onFiltersChange({ ...filters, search: search || undefined });
      }
    }, 300);
    return () => clearTimeout(timer);
  }, [search, filters, onFiltersChange]);

  const selectedCategoryIds = filters.category_ids ?? [];

  const handleCategoryToggle = (categoryId: string) => {
    const newIds = selectedCategoryIds.includes(categoryId)
      ? selectedCategoryIds.filter((id) => id !== categoryId)
      : [...selectedCategoryIds, categoryId];

    onFiltersChange({
      ...filters,
      category_ids: newIds.length > 0 ? newIds : undefined,
    });
  };

  const handleBankChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    onFiltersChange({
      ...filters,
      bank: value || undefined,
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
    (filters.category_ids && filters.category_ids.length > 0) ||
    filters.date_from ||
    filters.date_to ||
    filters.is_income !== undefined ||
    filters.bank;

  const getCategoryButtonLabel = () => {
    if (selectedCategoryIds.length === 0) {
      return "Todas las categorías";
    }
    if (selectedCategoryIds.length === 1) {
      const cat = categories.find((c) => c.id === selectedCategoryIds[0]);
      return cat ? getCategoryDisplayName(cat) : "1 categoría";
    }
    return `${selectedCategoryIds.length} categorías`;
  };

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

        {/* Category multi-select */}
        <div className="relative" ref={categoryDropdownRef}>
          <button
            type="button"
            onClick={() => setCategoryDropdownOpen(!categoryDropdownOpen)}
            className="h-9 min-w-[180px] rounded-md border border-input bg-background px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring flex items-center justify-between gap-2"
          >
            <span className="truncate">{getCategoryButtonLabel()}</span>
            <ChevronDown className="h-4 w-4 shrink-0 text-gray-400" />
          </button>
          {categoryDropdownOpen && (
            <div className="absolute top-full left-0 mt-1 w-64 max-h-60 overflow-y-auto bg-white border border-gray-200 rounded-md shadow-lg z-50">
              {categories.map((cat) => {
                const isSelected = selectedCategoryIds.includes(cat.id);
                const IconComponent =
                  ICON_MAP[cat.icon || "circle-dot"] || CircleDot;
                const color = getCategoryColor(cat);
                return (
                  <button
                    key={cat.id}
                    type="button"
                    onClick={() => handleCategoryToggle(cat.id)}
                    className="w-full px-3 py-2 text-left text-sm hover:bg-gray-100 flex items-center gap-2"
                  >
                    <div
                      className={`w-4 h-4 border rounded flex items-center justify-center ${
                        isSelected
                          ? "bg-primary border-primary"
                          : "border-gray-300"
                      }`}
                    >
                      {isSelected && <Check className="h-3 w-3 text-white" />}
                    </div>
                    <IconComponent size={16} style={{ color }} />
                    <span>{getCategoryDisplayName(cat)}</span>
                  </button>
                );
              })}
            </div>
          )}
        </div>

        {/* Bank filter */}
        {banks.length > 0 && (
          <select
            value={filters.bank ?? ""}
            onChange={handleBankChange}
            className="h-9 rounded-md border border-input bg-background px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
          >
            <option value="">Todos los bancos</option>
            {banks.map((bank) => (
              <option key={bank} value={bank}>
                {bank}
              </option>
            ))}
          </select>
        )}
      </div>

      {/* Date range row */}
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
