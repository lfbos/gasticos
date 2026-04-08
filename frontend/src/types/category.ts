/**
 * Category types matching backend API responses.
 */

/**
 * System category keys.
 * These match the backend SystemCategory enum.
 */
export type SystemCategoryKey =
  | "housing"
  | "groceries"
  | "restaurants"
  | "transportation"
  | "health"
  | "education"
  | "entertainment"
  | "clothing"
  | "financial"
  | "technology"
  | "subscriptions"
  | "other";

/**
 * Category from the API.
 */
export interface Category {
  id: string;
  name: string;
  key: string | null;
  icon: string | null;
  color: string | null;
  is_system: boolean;
}

/**
 * Request to create a new category.
 */
export interface CreateCategoryRequest {
  name: string;
  icon?: string;
  color?: string;
}

/**
 * Request to update a category.
 */
export interface UpdateCategoryRequest {
  name?: string;
  icon?: string;
  color?: string;
}

/**
 * Request to trigger categorization.
 */
export interface TriggerCategorizationRequest {
  transaction_ids?: string[];
  batch_size?: number;
}

/**
 * Response from categorization trigger.
 */
export interface TriggerCategorizationResponse {
  message: string;
  job_queued: boolean;
}

/**
 * Localized category names for Colombian Spanish.
 */
export const CATEGORY_NAMES_ES: Record<SystemCategoryKey, string> = {
  housing: "Vivienda",
  groceries: "Mercado",
  restaurants: "Restaurantes",
  transportation: "Transporte",
  health: "Salud",
  education: "Educación",
  entertainment: "Entretenimiento",
  clothing: "Ropa",
  financial: "Financiero",
  technology: "Tecnología",
  subscriptions: "Suscripciones",
  other: "Otros",
};

/**
 * Category icons (using Lucide icon names).
 */
export const CATEGORY_ICONS: Record<SystemCategoryKey, string> = {
  housing: "home",
  groceries: "shopping-cart",
  restaurants: "utensils",
  transportation: "car",
  health: "heart-pulse",
  education: "graduation-cap",
  entertainment: "gamepad",
  clothing: "shirt",
  financial: "landmark",
  technology: "laptop",
  subscriptions: "repeat",
  other: "circle-dot",
};

/**
 * Category colors (hex).
 */
export const CATEGORY_COLORS: Record<SystemCategoryKey, string> = {
  housing: "#8B5CF6",
  groceries: "#10B981",
  restaurants: "#F59E0B",
  transportation: "#3B82F6",
  health: "#EF4444",
  education: "#6366F1",
  entertainment: "#EC4899",
  clothing: "#F97316",
  financial: "#14B8A6",
  technology: "#6B7280",
  subscriptions: "#8B5CF6",
  other: "#9CA3AF",
};

/**
 * Get the display name for a category.
 */
export function getCategoryDisplayName(category: Category): string {
  if (category.key && category.key in CATEGORY_NAMES_ES) {
    return CATEGORY_NAMES_ES[category.key as SystemCategoryKey];
  }
  return category.name;
}

/**
 * Get the color for a category.
 */
export function getCategoryColor(category: Category): string {
  if (category.color) {
    return category.color;
  }
  if (category.key && category.key in CATEGORY_COLORS) {
    return CATEGORY_COLORS[category.key as SystemCategoryKey];
  }
  return CATEGORY_COLORS.other;
}

/**
 * Get the icon for a category.
 */
export function getCategoryIcon(category: Category): string {
  if (category.icon) {
    return category.icon;
  }
  if (category.key && category.key in CATEGORY_ICONS) {
    return CATEGORY_ICONS[category.key as SystemCategoryKey];
  }
  return CATEGORY_ICONS.other;
}
