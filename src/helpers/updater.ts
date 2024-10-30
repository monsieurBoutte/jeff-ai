import { check } from '@tauri-apps/plugin-updater';
import { ask } from '@tauri-apps/plugin-dialog';
import { relaunch } from '@tauri-apps/plugin-process';
import dedent from 'dedent';

export async function checkForAppUpdates() {
  const update = await check();

  if (update?.available) {
    const message = dedent`
    An update to ${update.version} is available!
    Release notes: ${update.body}
  `;

    const yes = await ask(message, {
      title: 'Update Now!',
      kind: 'info',
      okLabel: 'Update',
      cancelLabel: 'Cancel'
    });

    if (yes) {
      await update.downloadAndInstall();
      await relaunch();
    }
  }
}
