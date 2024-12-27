import { useEffect } from 'react';
import { Route, Switch } from 'wouter';

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

  return (
    <ThemeProvider>
      <SidebarProvider>
        <div className="flex h-screen w-full overflow-hidden">
          <AppSidebar />
          <main className="flex-1 flex flex-col w-full min-w-0">
            <SidebarTrigger />
            <section className="flex-1 w-full p-2 overflow-auto">
              <Router />
            </section>
            <Toaster />
          </main>
        </div>
      </SidebarProvider>
    </ThemeProvider>
  );
}

export default App;
