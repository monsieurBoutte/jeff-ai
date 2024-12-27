import {
  ListTodo,
  Home,
  PenLine,
  Search,
  Settings,
  CassetteTape,
  LogOut,
  LogIn,
  UserPlus
} from 'lucide-react';
import { Link, useLocation } from 'wouter';
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem
} from '@/components/ui/sidebar';
import { cn } from '@/lib/utils';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { ModeToggle } from '@/components/mode-toggle';

// Menu items.
const items = [
  {
    title: 'Home',
    url: '/',
    icon: Home
  },
  {
    title: 'Refinements',
    url: '/refinements',
    icon: PenLine
  },
  {
    title: 'Tasks',
    url: '/tasks',
    icon: ListTodo
  },
  {
    title: 'Recordings',
    url: '/recordings',
    icon: CassetteTape
  },
  {
    title: 'Search',
    url: '/search',
    icon: Search
  },
  {
    title: 'Settings',
    url: '/settings',
    icon: Settings
  }
];

export function AppSidebar() {
  const [location] = useLocation();
  const { logout, login, register, isAuthenticated, isLoading } =
    KindeAuth.useKindeAuth();

  return (
    <Sidebar>
      <SidebarContent className="flex flex-col h-full">
        <SidebarGroup>
          <SidebarGroupLabel>Jeff AI</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild>
                    <Link
                      to={item.url}
                      className={cn(
                        'hover:bg-gray-100 dark:hover:bg-gray-800/25',
                        location === item.url &&
                          'text-indigo-600 bg-gray-100 dark:bg-gray-800/25'
                      )}
                    >
                      <item.icon />
                      <span>{item.title}</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <div className="mt-auto px-2 flex justify-between">
          <SidebarMenu>
            {!isLoading && !isAuthenticated ? (
              <>
                <SidebarMenuItem>
                  <SidebarMenuButton
                    onClick={() => login()}
                    className="hover:bg-gray-100 dark:hover:bg-gray-800/25 text-gray-700 dark:text-gray-200"
                  >
                    <LogIn />
                    <span>Log In</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
                <SidebarMenuItem>
                  <SidebarMenuButton
                    onClick={() => register()}
                    className="hover:bg-gray-100 dark:hover:bg-gray-800/25 text-gray-700 dark:text-gray-200"
                  >
                    <UserPlus />
                    <span>Register</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              </>
            ) : isAuthenticated ? (
              <SidebarMenuItem>
                <SidebarMenuButton
                  onClick={() => logout()}
                  className="hover:bg-gray-100 dark:hover:bg-gray-800/25 text-gray-700 dark:text-gray-200"
                >
                  <LogOut />
                  <span>Log Out</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            ) : null}
          </SidebarMenu>
          <ModeToggle />
        </div>
      </SidebarContent>
    </Sidebar>
  );
}
