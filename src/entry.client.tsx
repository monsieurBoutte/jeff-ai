import React from 'react';
import ReactDOM from 'react-dom/client';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import { HydratedRouter } from 'react-router/dom';

// todo: make sure we add the production URIs into kinde's application details within their portal
ReactDOM.hydrateRoot(
  document,
  <React.StrictMode>
    <KindeAuth.KindeProvider
      clientId="98c15f1315d845e69890b864f3fb842e"
      domain="https://jeffai.kinde.com"
      logoutUri={window.location.origin}
      redirectUri={window.location.origin}
      isDangerouslyUseLocalStorage={process.env.NODE_ENV === 'development'}
    >
      <HydratedRouter />
    </KindeAuth.KindeProvider>
  </React.StrictMode>
);
