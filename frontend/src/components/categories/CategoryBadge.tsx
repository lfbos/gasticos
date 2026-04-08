/**
 * CategoryBadge component for displaying a category with icon and color.
 */

import {
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
import type { Category } from "@/types";
import { getCategoryDisplayName, getCategoryColor } from "@/types/category";

interface CategoryBadgeProps {
  category: Category;
  size?: "sm" | "md" | "lg";
  showIcon?: boolean;
  className?: string;
}

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

const SIZE_CLASSES = {
  sm: "px-2 py-0.5 text-xs",
  md: "px-2.5 py-1 text-sm",
  lg: "px-3 py-1.5 text-base",
};

const ICON_SIZES = {
  sm: 12,
  md: 14,
  lg: 16,
};

export function CategoryBadge({
  category,
  size = "md",
  showIcon = true,
  className = "",
}: CategoryBadgeProps) {
  const color = getCategoryColor(category);
  const displayName = getCategoryDisplayName(category);

  // Get the icon component
  const iconName = category.icon || "circle-dot";
  const IconComponent = ICON_MAP[iconName] || CircleDot;

  // Create styles with the category color
  const badgeStyle = {
    backgroundColor: `${color}20`, // 20% opacity
    color: color,
    borderColor: `${color}40`, // 40% opacity border
  };

  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full border font-medium ${SIZE_CLASSES[size]} ${className}`}
      style={badgeStyle}
    >
      {showIcon && <IconComponent size={ICON_SIZES[size]} />}
      {displayName}
    </span>
  );
}
