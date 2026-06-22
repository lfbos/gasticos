/**
 * Filters for analytics dashboard.
 */

import { useState, useEffect, useRef } from "react";
import { format } from "date-fns";
import type { DateRange } from "react-day-picker";
import {
  ChevronDown,
  Check,
  X,
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
import { DateRangePicker } from "@/components/ui/date-range-picker";
import { Button } from "@/components/ui/button";
import type { AnalyticsQuery } from "@/types/analytics";
import type { Category } from "@/types";
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

interface AnalyticsFiltersProps {
  filters: AnalyticsQuery;
  onFiltersChange: (filters: AnalyticsQuery) => void;
  categories: Category[];
  className?: string;
}

// Parse date string as local date (not UTC)
function parseLocalDate(dateStr: string): Date {
  const [year, month, day] = dateStr.split("-").map(Number);
  return new Date(year, month - 1, day);
}

export function AnalyticsFilters({
  filters,
  onFiltersChange,
  categories,
  className = "",
}: AnalyticsFiltersProps) {
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

  // Convert string dates to DateRange
  const dateRange: DateRange | undefined =
    filters.start_date || filters.end_date
      ? {
          from: filters.start_date
            ? parseLocalDate(filters.start_date)
            : undefined,
          to: filters.end_date ? parseLocalDate(filters.end_date) : undefined,
        }
      : undefined;

  const selectedCategoryIds = filters.category_ids ?? [];

  const handleDateRangeChange = (range: DateRange | undefined) => {
    onFiltersChange({
      ...filters,
      start_date: range?.from ? format(range.from, "yyyy-MM-dd") : undefined,
      end_date: range?.to ? format(range.to, "yyyy-MM-dd") : undefined,
    });
  };

  const handleTypeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    onFiltersChange({
      ...filters,
      is_income: value === "" ? undefined : value === "income",
    });
  };

  const handleCategoryToggle = (categoryId: string) => {
    const newIds = selectedCategoryIds.includes(categoryId)
      ? selectedCategoryIds.filter((id) => id !== categoryId)
      : [...selectedCategoryIds, categoryId];

    onFiltersChange({
      ...filters,
      category_ids: newIds.length > 0 ? newIds : undefined,
    });
  };

  const clearFilters = () => {
    onFiltersChange({});
  };

  const hasActiveFilters =
    filters.start_date ||
    filters.end_date ||
    (filters.category_ids && filters.category_ids.length > 0) ||
    filters.is_income !== undefined;

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
    <div className={`flex flex-wrap items-center gap-3 ${className}`}>
      {/* Date range picker */}
      <DateRangePicker
        value={dateRange}
        onChange={handleDateRangeChange}
        className="w-[280px]"
      />

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
          <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
        </button>
        {categoryDropdownOpen && (
          <div className="absolute top-full left-0 mt-1 w-64 max-h-60 overflow-y-auto bg-popover border border-border rounded-md shadow-lg z-50">
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
                  className="w-full px-3 py-2 text-left text-sm hover:bg-accent flex items-center gap-2"
                >
                  <div
                    className={`w-4 h-4 border rounded flex items-center justify-center ${
                      isSelected
                        ? "bg-primary border-primary"
                        : "border-muted-foreground"
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

      {/* Clear filters */}
      {hasActiveFilters && (
        <Button
          variant="ghost"
          size="sm"
          onClick={clearFilters}
          className="text-muted-foreground"
        >
          <X className="h-4 w-4 mr-1" />
          Limpiar
        </Button>
      )}
    </div>
  );
}
