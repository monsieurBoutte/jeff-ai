import {
  ListTodo,
  Home,
  Blend,
  Search,
  Settings,
  CassetteTape
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
import { ModeToggle } from '@/components/mode-toggle';

import packageJson from '../../package.json';

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
    icon: Blend
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

        <div className="mt-auto py-3 px-4 flex justify-between items-center">
          <span className="text-gray-400 dark:text-gray-700 text-sm">
            Version {packageJson.version}
          </span>
          <ModeToggle />
        </div>
      </SidebarContent>
    </Sidebar>
  );
}
