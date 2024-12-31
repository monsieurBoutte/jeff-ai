import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { LogOut, LogIn, UserPlus } from 'lucide-react';

import { Button } from '@/components/ui/button';

export default function Settings() {
  const { logout, isAuthenticated } = KindeAuth.useKindeAuth();

  return (
    <div>
      <div className="flex flex-row justify-between items-center">
        <h1 className="text-4xl font-bold text-gray-800 dark:text-white">
          Settings
        </h1>
      </div>
      <div className="mt-4">
        {isAuthenticated && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => logout()}
            className="hover:bg-gray-100 dark:hover:bg-gray-800/25 text-gray-700 dark:text-gray-200"
          >
            <LogOut />
            <span>Log Out</span>
          </Button>
        )}
      </div>
    </div>
  );
}
