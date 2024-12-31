import { useEffect, useCallback, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { listen } from '@tauri-apps/api/event';
import { register } from '@tauri-apps/plugin-global-shortcut';

import { DocumentEditor } from '@/components/document-editor';

interface TranscriptionEvent {
  payload: string;
}

export default function Home() {
  const [transcription, setTranscription] = useState<string>('');

  useEffect(() => {
    const unlisten = listen(
      'transcription-complete',
      (event: TranscriptionEvent) => {
        console.log('transcription-complete', event);
        setTranscription(`<p>${event.payload}</p>`);
      }
    );

    // Clean up the listener when the component unmounts
    return () => {
      unlisten.then((unlistenFn) => unlistenFn());
    };
  }, []);

  const { isAuthenticated, getToken, getUser } = KindeAuth.useKindeAuth();

  const handleShortcut = useCallback(async () => {
    if (!isAuthenticated || !getToken || !getUser) {
      return;
    }

    try {
      const token = await getToken();
      await register('CommandOrControl+Shift+J', (event) => {
        console.log('event', event.state);
        if (event.state === 'Pressed') {
          invoke('start_recording');
        }

        if (event.state === 'Released') {
          invoke('stop_recording', { token, refine: true });
        }
      });
    } catch (error) {
      console.error('Error registering shortcut:', error);
    }
  }, [isAuthenticated, getToken]);

  useEffect(() => {
    handleShortcut();
  }, [handleShortcut]);

  const captureUser = useCallback(async () => {
    if (!isAuthenticated || !getToken || !getUser) {
      return;
    }
    const authUser = getUser();
    const token = await getToken();

    await invoke('capture_user', {
      token,
      authUser
    });
  }, [getToken, getUser, isAuthenticated]);

  useEffect(() => {
    captureUser();
  }, [captureUser]);

  return (
    <div className="flex flex-col h-full w-full gap-4">
      <DocumentEditor content={transcription} />
    </div>
  );
}
