/**
 * Connect page for bank connections via Belvo Open Banking.
 */

import { useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { AlertCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { BelvoWidget, ConnectedBanks } from "@/components/belvo";
import { useAuth, useBelvo } from "@/hooks";
import type { BelvoWidgetSuccessEvent } from "@/types";

export function ConnectPage() {
  const { t } = useTranslation();
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  // Belvo state
  const {
    connectedBanks,
    accounts,
    isLoading: isBelvoLoading,
    isSyncing,
    error: belvoError,
    widgetToken,
    fetchWidgetToken,
    onWidgetSuccess,
    disconnectBank,
    syncBank,
    clearError: clearBelvoError,
  } = useBelvo();

  // Fetch widget token on mount
  useEffect(() => {
    fetchWidgetToken().catch(() => {
      // Error is handled by the hook
    });
  }, [fetchWidgetToken]);

  // Handle Belvo widget success
  const handleBelvoSuccess = useCallback(
    async (event: BelvoWidgetSuccessEvent) => {
      try {
        await onWidgetSuccess(event);
      } catch {
        // Error is handled by the hook
      }
    },
    [onWidgetSuccess],
  );

  return (
    <div className="min-h-screen bg-gray-100">
      {/* Header */}
      <header className="bg-white shadow">
        <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-6 sm:px-6 lg:px-8">
          <h1 className="text-3xl font-bold tracking-tight text-gray-900">
            Gasticos
          </h1>
          <div className="flex items-center gap-4">
            <Button variant="ghost" onClick={() => navigate("/dashboard")}>
              {t("dashboard.title")}
            </Button>
            <span className="text-sm text-gray-600">
              {t("auth.greeting", { name: user?.name })}
            </span>
            <Button variant="outline" onClick={logout}>
              {t("auth.logout")}
            </Button>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
        <div className="mb-6">
          <h2 className="text-2xl font-bold text-gray-900">
            {t("connect.title")}
          </h2>
          <p className="mt-1 text-sm text-gray-600">
            {t("belvo.connectBankDescription")}
          </p>
        </div>

        {/* Error display */}
        {belvoError && (
          <div className="mb-6 rounded-lg bg-red-50 p-4">
            <div className="flex">
              <AlertCircle className="h-5 w-5 text-red-400" />
              <div className="ml-3">
                <p className="text-sm text-red-800">{belvoError}</p>
              </div>
              <button
                onClick={clearBelvoError}
                className="ml-auto text-red-500 hover:text-red-700"
              >
                &times;
              </button>
            </div>
          </div>
        )}

        <div className="grid gap-6 md:grid-cols-2">
          {/* Connect new bank */}
          <Card>
            <CardHeader>
              <CardTitle>{t("belvo.connectNewBank")}</CardTitle>
              <CardDescription>
                {t("belvo.connectNewBankDescription")}
              </CardDescription>
            </CardHeader>
            <CardContent>
              {widgetToken ? (
                <BelvoWidget
                  accessToken={widgetToken}
                  onSuccess={handleBelvoSuccess}
                  disabled={isBelvoLoading}
                />
              ) : (
                <Button disabled className="w-full" size="lg">
                  {t("belvo.loadingWidget")}
                </Button>
              )}
            </CardContent>
          </Card>

          {/* Connected banks */}
          <ConnectedBanks
            banks={connectedBanks}
            accounts={accounts}
            isSyncing={isSyncing}
            onSync={syncBank}
            onDisconnect={disconnectBank}
            isLoading={isBelvoLoading}
          />
        </div>
      </main>
    </div>
  );
}
