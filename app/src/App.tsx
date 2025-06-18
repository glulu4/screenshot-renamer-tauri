import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import "./App.css";
import {listen} from '@tauri-apps/api/event'
import {
  warn,
  debug,
  trace,
  info,
  error,
  attachConsole,
  attachLogger,
} from '@tauri-apps/plugin-log';
// import {TrayIcon, TrayIconOptions} from '@tauri-apps/api/tray';
// import {Menu} from "@tauri-apps/api/menu";
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
    error(`❌ Failed to send notification: ${e}`);
  }
}

function App() {
  const [paused, setPaused] = useState(false);
  const [lastFileRenamed, setLastFileRenamed] = useState<string | null>(null);

  const blue = "rgb(0, 122, 255)"
  const gray = "rgb(174, 174, 178)"

  useEffect(() => {
    const notifyFileRename = async () => {
      if (!lastFileRenamed) {
        return;
      }
      debug(`📣 Notifying file rename: ${lastFileRenamed}`);
      await enqueueNotification("SnapName", lastFileRenamed);
    }
    notifyFileRename();
  }, [lastFileRenamed])

  useEffect(() => {
    let unlistenFn: (() => void) | null = null;

    const setupListener = async () => {
      try {
        unlistenFn = await listen<string>('screenshot-renamed', async (event) => {
          debug(`📣 Screenshot renamed: ${event.payload}`);
          setLastFileRenamed(event.payload);
        });
      } catch (err) {
        error(`Failed to setup event listener: ${err}`);
      }
    };

    setupListener();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, []);

  const toggleWatcher = async () => {
    console.log("Toggling watcher paused state", !paused);
    info("Toggling watcher paused state");
    await enqueueNotification("SnapName", `Watcher is now ${!paused ? "paused" : "active"}`);
    setPaused(!paused);
    await invoke("set_watcher_paused", {paused: !paused});
  };

  return (
    // <main style={{padding:5}}>
    //   <h3 style={{textAlign:'left'}}>SnapName</h3>
    //   <div>
    //     <label>
    //       <input type="checkbox" checked={paused} onChange={toggleWatcher} />
    //       Pause Screenshot Watcher
    //     </label>
    //   </div>
    //   <div></div>
    //   <button onClick={() => invoke("screenshot-renamed")}>Open Settings</button>
    // </main>
    <div className="menu-container">
      <h1 className="title">SnapName</h1>

      <div className="line-divider"></div>

      <div className="menu-item" onClick={toggleWatcher}>
        <span className="icon" style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',

          backgroundColor: paused ? gray : blue,
        }}>
          <img style={{color:"white", paddingLeft:1 }} src={paused ?  "/images/play.fill.svg" : "/images/pause.fill.svg"}/>
        </span>
        
        <p className="text">{`${paused ? "Start" : "Pause"} Screenshot Watcher`}</p>
      </div>


    </div>

  );
}

export default App;