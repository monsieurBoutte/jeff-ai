import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { LogOut, LogIn, UserPlus } from 'lucide-react';

import { Button } from '@/components/ui/button';

export default function Settings() {
  const { logout, login, register, isAuthenticated, isLoading } =
    KindeAuth.useKindeAuth();

  return (
    <div>
      <div className="flex flex-row justify-between items-center">
        <h1 className="text-4xl font-bold text-gray-800 dark:text-white">
          Settings
        </h1>
      </div>
      <div className="mt-4">
        {!isLoading && !isAuthenticated ? (
          <div className="flex flex-row gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => login()}
              className="hover:bg-gray-100 dark:hover:bg-gray-800/25 text-gray-700 dark:text-gray-200"
            >
              <LogIn />
              <span>Log In</span>
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => register()}
              className="hover:bg-gray-100 dark:hover:bg-gray-800/25 text-gray-700 dark:text-gray-200"
            >
              <UserPlus />
              <span>Register</span>
            </Button>
          </div>
        ) : isAuthenticated ? (
          <Button
            variant="outline"
            size="sm"
            onClick={() => logout()}
            className="hover:bg-gray-100 dark:hover:bg-gray-800/25 text-gray-700 dark:text-gray-200"
          >
            <LogOut />
            <span>Log Out</span>
          </Button>
        ) : null}
      </div>
    </div>
  );
}
