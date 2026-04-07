/**
 * Connected banks list component.
 *
 * Displays a list of banks connected via Belvo with sync status.
 */

import { useTranslation } from "react-i18next";
import {
  Building2,
  RefreshCw,
  Trash2,
  CheckCircle,
  AlertCircle,
  Clock,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type { BelvoAccount, ConnectedBank } from "@/types/belvo";
import { getBankDisplayName } from "@/types/belvo";
import { formatCurrency } from "@/lib/format";

interface ConnectedBanksProps {
  banks: ConnectedBank[];
  accounts: BelvoAccount[];
  isSyncing: Record<string, boolean>;
  onSync: (linkId: string) => void;
  onDisconnect: (linkId: string) => void;
  isLoading?: boolean;
}

export function ConnectedBanks({
  banks,
  accounts,
  isSyncing,
  onSync,
  onDisconnect,
  isLoading = false,
}: ConnectedBanksProps) {
  const { t } = useTranslation();

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>{t("belvo.connectedBanks")}</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-8">
            <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        </CardContent>
      </Card>
    );
  }

  if (banks.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>{t("belvo.connectedBanks")}</CardTitle>
          <CardDescription>{t("belvo.noConnectedBanks")}</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <Building2 className="h-12 w-12 text-muted-foreground mb-4" />
            <p className="text-sm text-muted-foreground">
              {t("belvo.connectBankDescription")}
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("belvo.connectedBanks")}</CardTitle>
        <CardDescription>
          {t("belvo.connectedBanksCount", { count: banks.length })}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {banks.map((bank) => {
          const bankAccounts = accounts.filter(
            (acc) => acc.link_id === bank.id,
          );
          const totalBalance = bankAccounts.reduce(
            (sum, acc) => sum + (acc.balance_current ?? 0),
            0,
          );

          return (
            <div
              key={bank.id}
              className="flex items-start justify-between p-4 rounded-lg border bg-card"
            >
              <div className="flex-1">
                <div className="flex items-center gap-2">
                  <Building2 className="h-5 w-5 text-primary" />
                  <h4 className="font-medium">
                    {bank.institution_name ||
                      getBankDisplayName(bank.institution)}
                  </h4>
                  <StatusBadge status={bank.status} />
                </div>

                {bankAccounts.length > 0 && (
                  <div className="mt-2 space-y-1">
                    {bankAccounts.map((account) => (
                      <div
                        key={account.id}
                        className="flex items-center justify-between text-sm text-muted-foreground"
                      >
                        <span>
                          {account.name ||
                            t("belvo.accountTypes." + account.account_type)}
                          {account.number_masked &&
                            ` (${account.number_masked})`}
                        </span>
                        <span className="font-mono">
                          {formatCurrency(account.balance_current ?? 0)}
                        </span>
                      </div>
                    ))}
                    <div className="flex items-center justify-between text-sm font-medium pt-2 border-t">
                      <span>{t("belvo.totalBalance")}</span>
                      <span className="font-mono">
                        {formatCurrency(totalBalance)}
                      </span>
                    </div>
                  </div>
                )}

                {bank.last_synced_at && (
                  <p className="text-xs text-muted-foreground mt-2">
                    {t("belvo.lastSync")}:{" "}
                    {formatRelativeTime(bank.last_synced_at)}
                  </p>
                )}
              </div>

              <div className="flex items-center gap-2 ml-4">
                <Button
                  variant="outline"
                  size="icon"
                  onClick={() => onSync(bank.id)}
                  disabled={isSyncing[bank.id] || bank.status !== "valid"}
                  title={t("belvo.sync")}
                >
                  <RefreshCw
                    className={`h-4 w-4 ${isSyncing[bank.id] ? "animate-spin" : ""}`}
                  />
                </Button>
                <Button
                  variant="outline"
                  size="icon"
                  onClick={() => onDisconnect(bank.id)}
                  title={t("belvo.disconnect")}
                  className="text-destructive hover:text-destructive"
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}

function StatusBadge({ status }: { status: string }) {
  const { t } = useTranslation();

  const config = {
    valid: {
      icon: CheckCircle,
      className: "text-green-600 bg-green-100",
      label: t("belvo.status.valid"),
    },
    invalid: {
      icon: AlertCircle,
      className: "text-red-600 bg-red-100",
      label: t("belvo.status.invalid"),
    },
    token_required: {
      icon: AlertCircle,
      className: "text-yellow-600 bg-yellow-100",
      label: t("belvo.status.tokenRequired"),
    },
    unconfirmed: {
      icon: Clock,
      className: "text-blue-600 bg-blue-100",
      label: t("belvo.status.unconfirmed"),
    },
  }[status] ?? {
    icon: Clock,
    className: "text-gray-600 bg-gray-100",
    label: status,
  };

  const Icon = config.icon;

  return (
    <span
      className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${config.className}`}
    >
      <Icon className="h-3 w-3" />
      {config.label}
    </span>
  );
}

function formatRelativeTime(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMins < 1) return "justo ahora";
  if (diffMins < 60) return `hace ${diffMins} min`;
  if (diffHours < 24) return `hace ${diffHours} h`;
  if (diffDays < 7) return `hace ${diffDays} d`;

  return date.toLocaleDateString("es-CO", {
    day: "numeric",
    month: "short",
  });
}
