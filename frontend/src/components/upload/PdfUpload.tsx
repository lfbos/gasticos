/**
 * PDF Upload component for bank statements.
 */

import { useCallback, useRef, useState } from "react";
import {
  Upload,
  FileText,
  CheckCircle,
  AlertCircle,
  Loader2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useUpload } from "@/hooks";

interface PdfUploadProps {
  onUploadComplete?: () => void;
}

export function PdfUpload({ onUploadComplete }: PdfUploadProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [password, setPassword] = useState("");

  const { isUploading, error, result, uploadPdf, clearError, clearResult } =
    useUpload();

  const handleFileSelect = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) {
        if (file.type !== "application/pdf") {
          return;
        }
        setSelectedFile(file);
        clearError();
        clearResult();
      }
    },
    [clearError, clearResult],
  );

  const handleDrop = useCallback(
    (e: React.DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      const file = e.dataTransfer.files?.[0];
      if (file && file.type === "application/pdf") {
        setSelectedFile(file);
        clearError();
        clearResult();
      }
    },
    [clearError, clearResult],
  );

  const handleDragOver = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
  }, []);

  const handleUpload = useCallback(async () => {
    if (!selectedFile || !password) return;

    try {
      await uploadPdf(selectedFile, password);
      // Clear form on success
      setSelectedFile(null);
      setPassword("");
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
      onUploadComplete?.();
    } catch {
      // Error is handled by hook
    }
  }, [selectedFile, password, uploadPdf, onUploadComplete]);

  const handleClear = useCallback(() => {
    setSelectedFile(null);
    setPassword("");
    clearError();
    clearResult();
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  }, [clearError, clearResult]);

  return (
    <div className="space-y-4">
      {/* Drop zone */}
      <div
        onClick={() => fileInputRef.current?.click()}
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        className={`
          border-2 border-dashed rounded-lg p-8 text-center cursor-pointer
          transition-colors duration-200
          ${selectedFile ? "border-green-500 bg-green-50" : "border-gray-300 hover:border-gray-400"}
        `}
      >
        <input
          ref={fileInputRef}
          type="file"
          accept=".pdf"
          onChange={handleFileSelect}
          className="hidden"
        />

        {selectedFile ? (
          <div className="flex flex-col items-center gap-2">
            <FileText className="h-12 w-12 text-green-600" />
            <p className="font-medium text-green-700">{selectedFile.name}</p>
            <p className="text-sm text-green-600">
              {(selectedFile.size / 1024).toFixed(1)} KB
            </p>
          </div>
        ) : (
          <div className="flex flex-col items-center gap-2">
            <Upload className="h-12 w-12 text-gray-400" />
            <p className="font-medium text-gray-700">
              Arrastra tu extracto PDF aquí
            </p>
            <p className="text-sm text-gray-500">o haz clic para seleccionar</p>
            <p className="text-xs text-gray-400 mt-2">
              Bancolombia, Nequi o Nu
            </p>
          </div>
        )}
      </div>

      {/* Password input */}
      {selectedFile && (
        <div className="space-y-2">
          <Label htmlFor="pdf-password">Contraseña del PDF</Label>
          <Input
            id="pdf-password"
            type="password"
            placeholder="Ingresa la contraseña del extracto"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            disabled={isUploading}
          />
          <p className="text-xs text-gray-500">
            La contraseña suele ser tu número de documento
          </p>
        </div>
      )}

      {/* Error message */}
      {error && (
        <div className="flex items-center gap-2 p-3 bg-red-50 border border-red-200 rounded-lg">
          <AlertCircle className="h-5 w-5 text-red-500 flex-shrink-0" />
          <p className="text-sm text-red-700">{error}</p>
        </div>
      )}

      {/* Success message */}
      {result && (
        <div className="flex items-start gap-2 p-3 bg-green-50 border border-green-200 rounded-lg">
          <CheckCircle className="h-5 w-5 text-green-500 flex-shrink-0 mt-0.5" />
          <div className="text-sm text-green-700">
            <p className="font-medium">Extracto procesado correctamente</p>
            <p>Banco: {result.bank}</p>
            <p>
              {result.transactions_inserted} transacciones importadas
              {result.transactions_skipped > 0 &&
                `, ${result.transactions_skipped} duplicadas`}
            </p>
          </div>
        </div>
      )}

      {/* Actions */}
      {selectedFile && (
        <div className="flex gap-2">
          <Button
            onClick={handleUpload}
            disabled={!password || isUploading}
            className="flex-1"
          >
            {isUploading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Procesando...
              </>
            ) : (
              <>
                <Upload className="mr-2 h-4 w-4" />
                Subir extracto
              </>
            )}
          </Button>
          <Button
            variant="outline"
            onClick={handleClear}
            disabled={isUploading}
          >
            Cancelar
          </Button>
        </div>
      )}
    </div>
  );
}
