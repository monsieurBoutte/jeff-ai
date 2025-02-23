import { useEffect } from 'react';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { LogIn, UserPlus, Smile } from 'lucide-react';
import { Route, Switch } from 'wouter';

import { Button } from '@/components/ui/button';
import { Toaster } from '@/components/ui/toaster';
import { AppSidebar } from '@/components/app-sidebar';
import { SidebarTrigger } from '@/components/ui/sidebar';
import { SidebarProvider } from '@/components/ui/sidebar';
import { ThemeProvider } from '@/components/theme-provider';
import { checkForAppUpdates } from '@/helpers/updater';
import Refinements from '@/pages/refinements';
import Home from '@/pages/home';
import Search from '@/pages/search';
import Settings from '@/pages/settings';
import Recordings from '@/pages/recordings';
import Tasks from '@/pages/tasks';

import './styles.css';

function Router() {
  return (
    <Switch>
      <Route path="/" component={Home} />
      <Route path="/refinements" component={Refinements} />
      <Route path="/tasks" component={Tasks} />
      <Route path="/recordings" component={Recordings} />
      <Route path="/search" component={Search} />
      <Route path="/settings" component={Settings} />
    </Switch>
  );
}

function App() {
  useEffect(() => {
    checkForAppUpdates();
  }, []);

  const { login, register, isAuthenticated } = KindeAuth.useKindeAuth();

  return (
    <ThemeProvider>
      <SidebarProvider>
        <div className="flex h-screen w-full overflow-hidden">
          <AppSidebar />
          <main className="flex-1 flex flex-col w-full min-w-0">
            <SidebarTrigger />
            <section className="flex-1 w-full p-2 overflow-auto">
              {!isAuthenticated ? (
                <div className="w-full h-full">
                  <div className="bg-white dark:bg-gray-900 rounded-lg p-6 shadow">
                    <div className="flex items-center gap-3 mb-2">
                      <Smile className="w-6 h-6 text-orange-500" />
                      <h2 className="text-2xl font-bold">Hello friend</h2>
                    </div>
                    <p className="text-gray-500 dark:text-gray-400 mb-4">
                      Please log in or register to continue
                    </p>
                    <div className="flex flex-col gap-2">
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
                  </div>
                </div>
              ) : (
                <Router />
              )}
            </section>
            <Toaster />
          </main>
        </div>
      </SidebarProvider>
    </ThemeProvider>
  );
}

export default App;
