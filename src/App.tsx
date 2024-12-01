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
        <AppSidebar />
        <main className="p-2 flex flex-col">
          <SidebarTrigger />
          <section className="flex-1 mt-1">
            <Router />
          </section>
          <Toaster />
        </main>
      </SidebarProvider>
    </ThemeProvider>
  );
}

export default App;
