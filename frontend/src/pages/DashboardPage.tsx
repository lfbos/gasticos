/**
 * Dashboard page - placeholder for protected content.
 */

import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Building2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useAuth } from '@/hooks';

export function DashboardPage() {
  const { t } = useTranslation();
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  return (
    <div className="min-h-screen bg-gray-100">
      <header className="bg-white shadow">
        <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-6 sm:px-6 lg:px-8">
          <h1 className="text-3xl font-bold tracking-tight text-gray-900">
            Gasticos
          </h1>
          <div className="flex items-center gap-4">
            <span className="text-sm text-gray-600">
              {t('auth.greeting', { name: user?.name })}
            </span>
            <Button variant="outline" onClick={logout}>
              {t('auth.logout')}
            </Button>
          </div>
        </div>
      </header>
      <main>
        <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
          <div className="rounded-lg bg-white p-6 shadow">
            <h2 className="text-xl font-semibold text-gray-800">
              {t('connect.title')}
            </h2>
            <p className="mt-2 text-gray-600">
              {t('connect.description')}
            </p>
            <Button
              className="mt-4"
              onClick={() => navigate('/connect')}
            >
              <Building2 className="mr-2 h-4 w-4" />
              {t('belvo.connectBank')}
            </Button>
          </div>
        </div>
      </main>
    </div>
  );
}
