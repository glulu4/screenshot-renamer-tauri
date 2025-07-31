import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import "./App.css";
import {listen} from '@tauri-apps/api/event'
import {
  warn,
  debug,
  info,
  error as logError,
} from '@tauri-apps/plugin-log';

import {enqueueNotification, notifyFreeLimit} from "./initNotifications";
import {useLiveUser} from "./hooks/useLiveUser";

import {message, open} from '@tauri-apps/plugin-dialog';
import {ask} from '@tauri-apps/plugin-dialog';


function App() {
  const [paused, setPaused] = useState(false);
  // const {deviceId} = useDeviceId();
  const [lastFileRenamed, setLastFileRenamed] = useState<string | null>(null);

  const {userDevice, deviceId} = useLiveUser();
  const hasActiveSubscription = userDevice?.subscriptionStatus === "active";

  info(`🔄 Live user data: ${JSON.stringify(userDevice)}`);

  const firstTimeUser:boolean = !userDevice?.stripeCustomerId
  const userEmail = userDevice?.email || "Not provided";

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
    let unlistenRename: (() => void) | null = null;
    let unlistenQuota: (() => void) | null = null;

    const setupListeners = async () => {
      try {
        unlistenRename = await listen<string>('screenshot-renamed', async (event) => {
          debug(`📣 Screenshot renamed: ${event.payload}`);
          setLastFileRenamed(event.payload);
        });

        unlistenQuota = await listen<string>('quota-exceeded', async (event) => {
          warn(`🚫 Quota exceeded for device: ${event.payload}`);
          alert("🚫 Your free tier quota is used up. Upgrade to continue.");
          await notifyFreeLimit();
        });

      } catch (err) {
        logError(`Failed to setup event listeners: ${err}`);
      }
    };

    setupListeners();

    return () => {
      if (unlistenRename) unlistenRename();
      if (unlistenQuota) unlistenQuota();
    };
  }, []);
  



  const toggleWatcher = async () => {
    console.log("Toggling watcher paused state", !paused);
    info("Toggling watcher paused state");
    await enqueueNotification("SnapName", `Watcher is now ${!paused ? "paused" : "active"}`);
    setPaused(!paused);
    await invoke("set_watcher_paused", {paused: !paused});
  };

  async function selectFolder() {

    info("Selecting folder to watch");

    

    const folder = await open({
      directory: true,
      multiple: false,
      title: "Select Folder to Watch",
      filters: [{
        name: "Folders",
        extensions: ["*"],
      }],
    })
    // const path = await invoke("select_folder");
    // if (!path) {
    //   warn("No folder selected");
    //   return;
    // }
    // info(`📂 Folder selected: ${path}`);

    // info("📂 Folder selected for watching");
  }


  return (

    <div style={{padding:5}}>
      <div style={{display: 'flex', alignItems: 'center', justifyContent: 'space-between',}}>
        <div style={{display: 'flex', alignItems: 'center', flexGrow: 1,}}>
          <h1 className="title">SnapName</h1>

          {hasActiveSubscription && <p style={{
            marginLeft: 1,
            color: "white",
            padding: "1px 4px",

            backgroundColor: blue,
            borderRadius: 6,
          }} className="pro-badge">Pro</p>}
        </div>
        <button className="folder-button" 
          onClick={async () => await selectFolder()}
        style={{
          width: 30,
          height: 30,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }} >
          <img
            src="/images/folder-dashed.svg"
            alt="SnapName Logo"
            style={{width: 18, height: 18, padding:5}}
          />
        </button>


      </div>

      <div className="line-divider"/>

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


      {firstTimeUser ?   <a 
      className="menu-item"
      style={{position:"absolute", bottom:10, left:10, fontSize:12, textDecoration:"none", color:blue, padding:"5px 10px", borderRadius:6, }}
      target="_blank"
        href={`https://buy.stripe.com/7sY8wP78R2k8h1fekI0oM00?client_reference_id=${deviceId}`}>
        Upgrade to Pro
      </a>
  :
      <a
        className="menu-item"
        style={{position: "absolute", bottom: 10, left: 10, fontSize: 12, textDecoration: "none", color: blue, padding: "5px 10px", borderRadius: 6, }}
        target="_blank"
          href={`https://billing.stripe.com/p/login/7sY8wP78R2k8h1fekI0oM00?prefilled_email=${encodeURIComponent(userEmail)}`}>
        Manage Subscription
      </a>}


    </div>

  );
}

export default App;








// function App() {
//   return (
//     <div style={{padding: 20, textAlign: 'center'}}>
//       <h1>SnapName</h1>
//       <p>Welcome to SnapName! This is a placeholder for the main application.</p>
//       <p>Please run the Tauri application to use the full functionality.</p>
//     </div>
//   );
// }
// export default App;