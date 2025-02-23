import { useState, useEffect } from 'react';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { invoke } from '@tauri-apps/api/core';

export type UserSettingsResponse = {
  id: string;
  userId: string;
  lat: number | null;
  lon: number | null;
  city: string | null;
  state: string | null;
  country: string | null;
  units: 'imperial' | 'metric';
  language: string;
};

export function useUserSettings() {
  const { logout, login, isAuthenticated, getToken } = KindeAuth.useKindeAuth();
  const [settings, setSettings] = useState<UserSettingsResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const fetchSettings = async () => {
      if (!isAuthenticated || !getToken) {
        setIsLoading(false);
        return;
      }

      try {
        setIsLoading(true);
        setError(null);
        const token = await getToken();
        const userSettings = await invoke<UserSettingsResponse>(
          'get_user_settings',
          { token }
        );

        setSettings(userSettings);
      } catch (err) {
        setError(
          err instanceof Error ? err : new Error('Failed to fetch settings')
        );
      } finally {
        setIsLoading(false);
      }
    };

    fetchSettings();
  }, [isAuthenticated, getToken]);

  return {
    logout,
    login,
    isAuthenticated,
    getToken,
    settings,
    isLoading,
    error,
    refetch: async () => {
      if (!isAuthenticated || !getToken) return;
      const token = await getToken();
      const userSettings = await invoke<UserSettingsResponse>(
        'get_user_settings',
        { token }
      );
      setSettings(userSettings);
    }
  };
}
