/**
 * Categories hook.
 * DOM-free for React Native reuse.
 */

import { useCallback, useEffect, useState } from "react";
import { categoriesApi } from "@/api/categories";
import { getApiErrorMessage } from "@/api/client";
import type {
  Category,
  CreateCategoryRequest,
  UpdateCategoryRequest,
  TriggerCategorizationResponse,
} from "@/types";

interface UseCategoriesReturn {
  // State
  categories: Category[];
  systemCategories: Category[];
  customCategories: Category[];
  isLoading: boolean;
  isCategorizing: boolean;
  error: string | null;

  // Actions
  fetchCategories: () => Promise<void>;
  createCategory: (data: CreateCategoryRequest) => Promise<Category>;
  updateCategory: (
    id: string,
    data: UpdateCategoryRequest,
  ) => Promise<Category>;
  deleteCategory: (id: string) => Promise<void>;
  triggerCategorization: (
    transactionIds?: string[],
  ) => Promise<TriggerCategorizationResponse>;
  getCategoryById: (id: string) => Category | undefined;
  clearError: () => void;
}

export function useCategories(): UseCategoriesReturn {
  const [categories, setCategories] = useState<Category[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isCategorizing, setIsCategorizing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Derived state
  const systemCategories = categories.filter((c) => c.is_system);
  const customCategories = categories.filter((c) => !c.is_system);

  const fetchCategories = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const data = await categoriesApi.getCategories();
      setCategories(data);
    } catch (err) {
      const message = getApiErrorMessage(err);
      setError(message);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const createCategory = useCallback(
    async (data: CreateCategoryRequest): Promise<Category> => {
      setError(null);
      try {
        const newCategory = await categoriesApi.createCategory(data);
        setCategories((prev) => [...prev, newCategory]);
        return newCategory;
      } catch (err) {
        const message = getApiErrorMessage(err);
        setError(message);
        throw new Error(message);
      }
    },
    [],
  );

  const updateCategory = useCallback(
    async (id: string, data: UpdateCategoryRequest): Promise<Category> => {
      setError(null);
      try {
        const updated = await categoriesApi.updateCategory(id, data);
        setCategories((prev) => prev.map((c) => (c.id === id ? updated : c)));
        return updated;
      } catch (err) {
        const message = getApiErrorMessage(err);
        setError(message);
        throw new Error(message);
      }
    },
    [],
  );

  const deleteCategory = useCallback(async (id: string): Promise<void> => {
    setError(null);
    try {
      await categoriesApi.deleteCategory(id);
      setCategories((prev) => prev.filter((c) => c.id !== id));
    } catch (err) {
      const message = getApiErrorMessage(err);
      setError(message);
      throw new Error(message);
    }
  }, []);

  const triggerCategorization = useCallback(
    async (
      transactionIds?: string[],
    ): Promise<TriggerCategorizationResponse> => {
      setIsCategorizing(true);
      setError(null);
      try {
        const result = await categoriesApi.triggerCategorization({
          transaction_ids: transactionIds,
        });
        return result;
      } catch (err) {
        const message = getApiErrorMessage(err);
        setError(message);
        throw new Error(message);
      } finally {
        setIsCategorizing(false);
      }
    },
    [],
  );

  const getCategoryById = useCallback(
    (id: string): Category | undefined => {
      return categories.find((c) => c.id === id);
    },
    [categories],
  );

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  // Fetch categories on mount
  useEffect(() => {
    fetchCategories();
  }, [fetchCategories]);

  return {
    categories,
    systemCategories,
    customCategories,
    isLoading,
    isCategorizing,
    error,
    fetchCategories,
    createCategory,
    updateCategory,
    deleteCategory,
    triggerCategorization,
    getCategoryById,
    clearError,
  };
}
