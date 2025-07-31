import {info, warn, error as logError} from '@tauri-apps/plugin-log';
import {
    isPermissionGranted,
    requestPermission,
    sendNotification
} from '@tauri-apps/plugin-notification'

async function checkPermission() {
  if (!(await isPermissionGranted())) {
    return (await requestPermission()) === 'granted'
  }
  return true
}

export async function enqueueNotification(title: string, body: string) {
  if (!(await checkPermission())) {
    warn("Notification permission not granted");
    return;
  }
  info(`🔔 Sending notification: ${title} - ${body}`);
  try {
    await sendNotification({title, body});
    info(`✅ Notification sent successfully`);
  } catch (e) {
    logError(`❌ Failed to send notification: ${e}`);
  }
}



export const notifyFreeLimit = async () => {
    if (!(await checkPermission())) {
        warn("Notification permission not granted");
        return;
    }
    info("🔔 Sending free limit notification");
    await enqueueNotification("SnapName", "You have reached your free tier limit. Upgrade to continue using SnapName.");
}