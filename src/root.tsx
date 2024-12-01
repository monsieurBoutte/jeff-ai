import {
  Links,
  Meta,
  Outlet,
  Scripts,
  ScrollRestoration,
  isRouteErrorResponse
} from 'react-router';
import { Route } from './+types/root';

import { AppSidebar } from './components/app-sidebar';
import { SidebarTrigger } from './components/ui/sidebar';
import { SidebarProvider } from './components/ui/sidebar';
import { ThemeProvider } from '@/components/theme-provider';
import './styles.css';

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <meta charSet="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <link rel="stylesheet" href="/styles.css" />
        <title>Jeff AI</title>
        <Meta />
        <Links />
      </head>
      <body>
        <ThemeProvider>
          <SidebarProvider>
            <AppSidebar />
            <main className="p-2 flex flex-col">
              <SidebarTrigger />
              <section className="flex-1 mt-1">{children}</section>
            </main>
          </SidebarProvider>
        </ThemeProvider>
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

export function ErrorBoundary({ error }: Route.ErrorBoundaryProps) {
  if (isRouteErrorResponse(error)) {
    return (
      <>
        <h1>
          {error.status} {error.statusText}
        </h1>
        <p>{error.data}</p>
      </>
    );
  } else if (error instanceof Error) {
    return (
      <div>
        <h1>Error</h1>
        <p>{error.message}</p>
        <p>The stack trace is:</p>
        <pre>{error.stack}</pre>
      </div>
    );
  } else {
    return <h1>Unknown Error</h1>;
  }
}

export default function Root() {
  return <Outlet />;
}
