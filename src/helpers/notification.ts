import {
  isPermissionGranted,
  requestPermission,
  sendNotification
} from '@tauri-apps/plugin-notification';

// this is just boilerplate code to send a notification
// adopt this as needed

export async function handleNotifications() {
  const hasPermission = await isPermissionGranted();

  if (!hasPermission) {
    const permission = await requestPermission();

    if (permission === 'granted') {
      sendNotification({
        title: 'Hello from JS!',
        body: 'This is a notification from JS and Rust'
      });
    } else {
      // permission denied — noop
    }
  } else {
    sendNotification({
      title: 'Hello from JavaScript!',
      body: 'This is a notification from JavaScript and Rust'
    });
  }
}
