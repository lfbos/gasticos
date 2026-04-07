/**
 * Layout for authentication pages (login, register).
 */

import type { ReactNode } from "react";

interface AuthLayoutProps {
  children: ReactNode;
}

export function AuthLayout({ children }: AuthLayoutProps) {
  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-100 px-4 py-12 sm:px-6 lg:px-8">
      <div className="w-full max-w-md space-y-8">
        <div className="text-center">
          <h1 className="text-3xl font-bold tracking-tight text-gray-900">
            Gasticos
          </h1>
          <p className="mt-2 text-sm text-gray-600">
            Analiza tus finanzas personales
          </p>
        </div>
        {children}
      </div>
    </div>
  );
}
