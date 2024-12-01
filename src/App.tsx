import { useEffect } from 'react';
import { Routes, Route } from 'react-router';

import { Toaster } from '@/components/ui/toaster';
import { checkForAppUpdates } from '@/helpers/updater';
import Refinements from '@/pages/refinements';
import Home from '@/pages/home';
import Search from '@/pages/search';
import Settings from '@/pages/settings';
import Recordings from '@/pages/recordings';
import Tasks from '@/pages/tasks';

function App() {
  useEffect(() => {
    checkForAppUpdates();
  }, []);

  return (
    <>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/refinements" element={<Refinements />} />
        <Route path="/tasks" element={<Tasks />} />
        <Route path="/recordings" element={<Recordings />} />
        <Route path="/search" element={<Search />} />
        <Route path="/settings" element={<Settings />} />
      </Routes>
      <Toaster />
    </>
  );
}

export default App;
