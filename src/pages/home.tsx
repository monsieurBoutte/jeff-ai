import { useEffect, useCallback, useState, useMemo } from 'react';
import { useMachine } from '@xstate/react';
import { invoke } from '@tauri-apps/api/core';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { listen } from '@tauri-apps/api/event';

import { modeMachine } from '@/machines/mode-machine';
import { MicrophoneToggle } from '@/components/microphone-toggle';
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

  // Format transcriptions into HTML content
  const editorContent = useMemo(
    () => transcriptions.map((t) => `<p>${t.transcript}</p>`).join(''),
    [transcriptions]
  );

  return (
    <div className="flex flex-col h-full w-full gap-4">
      <div className="flex flex-row justify-between items-center">
        <div className="flex gap-2">
          <MicrophoneToggle
            isActive={state.matches('recorder')}
            onClick={handleMicToggle}
          />
        </div>
      </div>
      <div className="w-[calc(100vw-1rem)] md:w-[calc(80vw-7rem)]">
        <DocumentEditor content={editorContent} />
      </div>
    </div>
  );
}
