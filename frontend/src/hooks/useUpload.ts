/**
 * Upload hook for PDF bank statements.
 * DOM-free for React Native reuse.
 */

import { useCallback, useState } from "react";
import { uploadApi, type UploadResponse } from "@/api/upload";
import { getApiErrorMessage } from "@/api/client";

interface UseUploadReturn {
  // State
  isUploading: boolean;
  error: string | null;
  result: UploadResponse | null;

  // Actions
  uploadPdf: (file: File, password: string) => Promise<UploadResponse>;
  clearError: () => void;
  clearResult: () => void;
}

export function useUpload(): UseUploadReturn {
  const [isUploading, setIsUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<UploadResponse | null>(null);

  const uploadPdf = useCallback(
    async (file: File, password: string): Promise<UploadResponse> => {
      setIsUploading(true);
      setError(null);
      setResult(null);
      try {
        const response = await uploadApi.uploadPdf(file, password);
        setResult(response);
        return response;
      } catch (err) {
        const message = getApiErrorMessage(err);
        setError(message);
        throw new Error(message);
      } finally {
        setIsUploading(false);
      }
    },
    [],
  );

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  const clearResult = useCallback(() => {
    setResult(null);
  }, []);

  return {
    isUploading,
    error,
    result,
    uploadPdf,
    clearError,
    clearResult,
  };
}
