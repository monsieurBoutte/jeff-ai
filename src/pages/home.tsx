import { useMachine } from '@xstate/react';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';

import { modeMachine } from '@/machines/mode-machine';
import { ModeToggle } from '@/components/mode-toggle';
import { MicrophoneToggle } from '@/components/microphone-toggle';
import { RewriteToggle } from '@/components/rewrite-toggle';
import { RewriteForm } from '@/components/rewrite-form';
import { Button } from '@/components/ui/button';

export default function Home() {
  const [state, send] = useMachine(modeMachine);

  const handleMicToggle = () => {
    send({ type: 'TOGGLE_RECORDER' });
  };

  const handleRewriteToggle = () => {
    send({ type: 'TOGGLE_REWRITE' });
  };

  const { isAuthenticated, login, logout, isLoading, register, user } =
    KindeAuth.useKindeAuth();

  console.log('user', user);

  return (
    <div>
      {!isLoading && !isAuthenticated && (
        <div className="flex gap-3 my-2">
          <Button variant="outline" onClick={() => login()} type="button">
            Log In
          </Button>
          <Button variant="outline" onClick={() => register()} type="button">
            Register
          </Button>
        </div>
      )}
      {isAuthenticated && (
        <Button variant="outline" onClick={() => logout()} type="button">
          Log Out
        </Button>
      )}
      <div className="flex flex-row justify-between items-center">
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
    </div>
  );
}
