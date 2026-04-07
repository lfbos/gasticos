/**
 * CategoryList component for displaying all categories.
 */

import { CategoryBadge } from "./CategoryBadge";
import type { Category } from "@/types";

interface CategoryListProps {
  categories: Category[];
  selectedCategoryId?: string | null;
  onSelectCategory?: (category: Category | null) => void;
  showSystemLabel?: boolean;
  className?: string;
}

export function CategoryList({
  categories,
  selectedCategoryId,
  onSelectCategory,
  showSystemLabel = false,
  className = "",
}: CategoryListProps) {
  const systemCategories = categories.filter((c) => c.is_system);
  const customCategories = categories.filter((c) => !c.is_system);

  const handleClick = (category: Category) => {
    if (!onSelectCategory) return;

    if (selectedCategoryId === category.id) {
      onSelectCategory(null); // Deselect
    } else {
      onSelectCategory(category);
    }
  };

  const renderCategories = (cats: Category[], label?: string) => (
    <div className="space-y-2">
      {label && (
        <h4 className="text-sm font-medium text-gray-500 mb-2">{label}</h4>
      )}
      <div className="flex flex-wrap gap-2">
        {cats.map((category) => (
          <button
            key={category.id}
            type="button"
            onClick={() => handleClick(category)}
            className={`transition-transform hover:scale-105 ${
              onSelectCategory ? "cursor-pointer" : "cursor-default"
            } ${
              selectedCategoryId === category.id
                ? "ring-2 ring-offset-2 ring-blue-500 rounded-full"
                : ""
            }`}
          >
            <CategoryBadge category={category} />
          </button>
        ))}
      </div>
    </div>
  );

  return (
    <div className={`space-y-4 ${className}`}>
      {showSystemLabel && customCategories.length > 0 ? (
        <>
          {renderCategories(systemCategories, "Categor\u00EDas del sistema")}
          {renderCategories(customCategories, "Mis categor\u00EDas")}
        </>
      ) : (
        renderCategories(categories)
      )}
    </div>
  );
}
