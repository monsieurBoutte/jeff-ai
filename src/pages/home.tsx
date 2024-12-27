import { useEffect, useCallback, useState } from 'react';
import { useMachine } from '@xstate/react';
import { invoke } from '@tauri-apps/api/core';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { listen } from '@tauri-apps/api/event';

import { modeMachine } from '@/machines/mode-machine';
import { MicrophoneToggle } from '@/components/microphone-toggle';
import { RewriteToggle } from '@/components/rewrite-toggle';
import { RewriteForm } from '@/components/rewrite-form';
import { DocumentEditor } from '@/components/document-editor';

interface TranscriptionEvent {
  payload: string;
}

interface Transcription {
  transcript: string;
  created_at: string;
}

export default function Home() {
  const [state, send] = useMachine(modeMachine);
  const [transcriptions, setTranscriptions] = useState<Array<Transcription>>(
    []
  );
  console.log('state', state.value);

  useEffect(() => {
    const unsubscribe = listen(
      'transcription-complete',
      (event: TranscriptionEvent) => {
        console.log('transcription-complete', event);
        setTranscriptions((prev) => [
          {
            transcript: event.payload,
            created_at: new Date().toISOString()
          },
          ...prev
        ]);
      }
    );

    // Clean up the listener when component unmounts
    return () => {
      unsubscribe.then((unlisten) => unlisten());
    };
  }, []);

  const handleMicToggle = useCallback(() => {
    send({ type: 'TOGGLE_RECORDER' });
    if (state.matches('recorder')) {
      invoke('stop_recording');
    } else {
      invoke('start_recording');
    }
  }, [send, state]);

  const handleRewriteToggle = useCallback(() => {
    send({ type: 'TOGGLE_REWRITE' });
  }, [send]);

  const { isAuthenticated, getToken } = KindeAuth.useKindeAuth();

  const captureUser = useCallback(async () => {
    if (!isAuthenticated || !getToken) {
      return;
    }
    const token = await getToken();

    await invoke('capture_user', {
      token
    });
  }, [getToken, isAuthenticated]);

  useEffect(() => {
    captureUser();
  }, [captureUser]);

  return (
    <div className="flex flex-col h-full gap-4">
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
        </div>
      </div>
      <div className="w-full bg-slate-300">
        <DocumentEditor />
        {/* {state.matches('rewrite') && <RewriteForm />} */}
      </div>
      {transcriptions.length > 0 && (
        <div className="backdrop-blur-sm bg-white/10 dark:bg-black/10 px-4 py-3 rounded-lg border border-gray-200 dark:border-gray-800 shadow-sm">
          <div className="flex flex-col gap-2">
            {[...transcriptions]
              .sort(
                (a, b) =>
                  new Date(b.created_at).getTime() -
                  new Date(a.created_at).getTime()
              )
              .map((transcription, index) => (
                <div key={transcription.created_at}>
                  <p className="select-text text-gray-700 dark:text-gray-200">
                    {transcription.transcript}
                  </p>
                  {index === transcriptions.length - 1 && (
                    <time className="select-none text-xs text-gray-300 dark:text-gray-400 mt-2 block font-extralight">
                      <span className="italic">latest entry @ </span>
                      {new Date(transcription.created_at).toLocaleString()}
                    </time>
                  )}
                </div>
              ))}
          </div>
        </div>
      )}
    </div>
  );
}
