/**
 * Upload API client.
 */

import axios from "axios";
import { storage } from "@/lib/storage";

const API_URL = import.meta.env.VITE_API_URL || "http://localhost:8080";

/**
 * Response from PDF upload.
 */
export interface UploadResponse {
  bank: string;
  transactions_parsed: number;
  transactions_inserted: number;
  transactions_skipped: number;
}

/**
 * Upload a bank statement PDF.
 */
export async function uploadPdf(
  file: File,
  password: string,
): Promise<UploadResponse> {
  const formData = new FormData();
  formData.append("file", file);

  const token = storage.getAccessToken();

  const response = await axios.post<UploadResponse>(
    `${API_URL}/api/v1/upload/pdf?password=${encodeURIComponent(password)}`,
    formData,
    {
      headers: {
        "Content-Type": "multipart/form-data",
        Authorization: token ? `Bearer ${token}` : undefined,
      },
    },
  );

  return response.data;
}

export const uploadApi = {
  uploadPdf,
};
