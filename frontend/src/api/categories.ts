/**
 * Categories API client.
 */

import { apiClient } from "./client";
import type {
  Category,
  CreateCategoryRequest,
  UpdateCategoryRequest,
  TriggerCategorizationRequest,
  TriggerCategorizationResponse,
} from "@/types";

/**
 * Get all categories (system + user custom).
 */
async function getCategories(): Promise<Category[]> {
  const response = await apiClient.get<Category[]>("/categories");
  return response.data;
}

/**
 * Get a single category by ID.
 */
async function getCategory(id: string): Promise<Category> {
  const response = await apiClient.get<Category>(`/categories/${id}`);
  return response.data;
}

/**
 * Create a new custom category.
 */
async function createCategory(data: CreateCategoryRequest): Promise<Category> {
  const response = await apiClient.post<Category>("/categories", data);
  return response.data;
}

/**
 * Update a custom category.
 */
async function updateCategory(
  id: string,
  data: UpdateCategoryRequest,
): Promise<Category> {
  const response = await apiClient.put<Category>(`/categories/${id}`, data);
  return response.data;
}

/**
 * Delete a custom category.
 */
async function deleteCategory(id: string): Promise<{ message: string }> {
  const response = await apiClient.delete<{ message: string }>(
    `/categories/${id}`,
  );
  return response.data;
}

/**
 * Trigger categorization of uncategorized transactions.
 */
async function triggerCategorization(
  data?: TriggerCategorizationRequest,
): Promise<TriggerCategorizationResponse> {
  const response = await apiClient.post<TriggerCategorizationResponse>(
    "/categories/categorize",
    data ?? {},
  );
  return response.data;
}

export const categoriesApi = {
  getCategories,
  getCategory,
  createCategory,
  updateCategory,
  deleteCategory,
  triggerCategorization,
};
