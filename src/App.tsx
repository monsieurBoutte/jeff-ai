import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ThemeProvider } from '@/components/theme-provider';

import './styles.css';
import { ModeToggle } from './components/mode-toggle';
import { Input } from './components/ui/input';
import { Button } from './components/ui/button';
import { checkForAppUpdates } from './helpers/updater';

function App() {
  const [greetMsg, setGreetMsg] = useState('');
  const [name, setName] = useState('');

  useEffect(() => {
    checkForAppUpdates();
  }, []);

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke('greet', { name }));
    // await invoke('greet', { name });
  }

  return (
    <ThemeProvider>
      <main className="bg-white dark:bg-black p-4 flex flex-col gap-4 max-w-md mx-auto">
        <div className="flex flex-row justify-between items-center">
          <h1 className="text-4xl font-bold text-gray-800 dark:text-white">
            Hello friend
          </h1>
          <ModeToggle />
        </div>
        <Input value={name} onChange={(e) => setName(e.target.value)} />
        <Button onClick={greet}>Greet</Button>
        {name && greetMsg && <p>{greetMsg}</p>}
      </main>
    </ThemeProvider>
  );
}

export default App;
