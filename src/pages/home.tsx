import { useEffect, useCallback, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { listen } from '@tauri-apps/api/event';

import { DocumentEditor } from '@/components/document-editor';

interface TranscriptionEvent {
  payload: string;
}

export default function Home() {
  const [transcription, setTranscription] = useState<string>('');

  useEffect(() => {
    const unsubscribe = listen(
      'transcription-complete',
      (event: TranscriptionEvent) => {
        console.log('transcription-complete', event);
        setTranscription(`<p>${event.payload}</p>`);
      }
    );

    // Clean up the listener when component unmounts
    return () => {
      unsubscribe.then((unlisten) => unlisten());
    };
  }, []);

  const { isAuthenticated, getToken, getUser } = KindeAuth.useKindeAuth();

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
