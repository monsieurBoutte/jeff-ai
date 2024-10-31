import { useEffect } from 'react';
import { useMachine } from '@xstate/react';

import { Toaster } from '@/components/ui/toaster';
import { ThemeProvider } from '@/components/theme-provider';
import { ModeToggle } from '@/components/mode-toggle';
import { MicrophoneToggle } from '@/components/microphone-toggle';
import { RewriteToggle } from '@/components/rewrite-toggle';
import { RewriteForm } from '@/components/rewrite-form';
import { checkForAppUpdates } from '@/helpers/updater';
import { modeMachine } from '@/machines/mode-machine';
import './styles.css';

function App() {
  const [state, send] = useMachine(modeMachine);

  useEffect(() => {
    checkForAppUpdates();
  }, []);

  const handleMicToggle = () => {
    send({ type: 'TOGGLE_RECORDER' });
  };

  const handleRewriteToggle = () => {
    send({ type: 'TOGGLE_REWRITE' });
  };

  return (
    <ThemeProvider>
      <main className="p-4 flex flex-col gap-4 max-w-md mx-auto">
        <div className="flex flex-row justify-between items-center">
          <h1 className="text-4xl font-bold text-gray-800 dark:text-white">
            Jeff AI
          </h1>
          <div className="flex gap-2">
            <RewriteToggle
              isActive={state.matches('rewrite')}
              onClick={handleRewriteToggle}
            />
            <MicrophoneToggle
              isActive={state.matches('recorder')}
              onClick={handleMicToggle}
            />
            <ModeToggle />
          </div>
        </div>
        {state.matches('rewrite') && <RewriteForm />}
      </main>
      <Toaster />
    </ThemeProvider>
  );
}

export default App;
