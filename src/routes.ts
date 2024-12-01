import { type RouteConfig, route } from '@react-router/dev/routes';

export default [
  route('/', './pages/home.tsx'),
  route('/refinements', './pages/refinements.tsx'),
  route('/tasks', './pages/tasks.tsx'),
  route('/recordings', './pages/recordings.tsx'),
  route('/search', './pages/search.tsx'),
  route('/settings', './pages/settings.tsx'),
  // * matches all URLs, the ? makes it optional so it will match / as well
  route('*?', 'catchall.tsx')
] satisfies RouteConfig;
