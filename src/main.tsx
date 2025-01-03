import React from 'react';
import ReactDOM from 'react-dom/client';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { invoke } from '@tauri-apps/api/core';
import App from './App';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <KindeAuth.KindeProvider
      clientId="98c15f1315d845e69890b864f3fb842e"
      domain="https://jeffai.kinde.com"
      logoutUri={window.location.origin}
      redirectUri={window.location.origin}
      isDangerouslyUseLocalStorage={process.env.NODE_ENV === 'development'}
      onRedirectCallback={async (userData) => {
        await invoke('set_user', { userData });
      }}
    >
      <App />
    </KindeAuth.KindeProvider>
  </React.StrictMode>
);
