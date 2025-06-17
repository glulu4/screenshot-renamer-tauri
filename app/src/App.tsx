import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import {listen, UnlistenFn} from '@tauri-apps/api/event'
import {
  isPermissionGranted,
  requestPermission,
  sendNotification
} from '@tauri-apps/plugin-notification'
// import {TrayIcon, TrayIconOptions} from '@tauri-apps/api/tray';
// import {Menu} from "@tauri-apps/api/menu";


function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  const [paused, setPaused] = useState(false);

  const toggleWatcher = async () => {
    setPaused(!paused);
    await invoke("set_watcher_paused", {paused: !paused});
  };


  async function checkPermission() {
    if (!(await isPermissionGranted())) {
      return (await requestPermission()) === 'granted'
    }
    return true
  }

  useEffect(() => {
    let unlisten: UnlistenFn
    listen<{name: string}>('screenshot-renamed', async event => {

      if (!(await checkPermission())) {
        return
      }
      sendNotification({
        title: 'Screenshot Renamed',
        body: `Saved as ${event.payload.name}`
      })
    }).then(f => {unlisten = f})

    return () => {unlisten && unlisten()}
  }, [])


  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main>
      <h1>SnapName</h1>

      <div>
        <label>
          <input type="checkbox" checked={paused} onChange={toggleWatcher} />
          Pause Screenshot Watcher
        </label>
      </div>

    </main>
  );
}

export default App;
