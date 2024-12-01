import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export default function Tasks() {
  const { isAuthenticated, getToken } = KindeAuth.useKindeAuth();

  const fetchTasks = useCallback(async () => {
    if (!isAuthenticated || !getToken) {
      return;
    }
    const token = await getToken();

    const response = await invoke('fetch_tasks', {
      token
    });
    console.log('response', response);
  }, [getToken, isAuthenticated]);

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  return (
    <div>
      <div className="flex flex-col justify-between items-center">
        <h1 className="text-4xl font-bold text-gray-800 dark:text-white">
          Tasks
        </h1>
      </div>
    </div>
  );
}
