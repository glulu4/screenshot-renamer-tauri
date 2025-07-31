import {invoke} from "@tauri-apps/api/core";
import {debug, error as logError} from "@tauri-apps/plugin-log";
import {useEffect, useState} from "react";


export const useDeviceId = () => {

    const [deviceId, setDeviceId] = useState<string | null>(null);

    useEffect(() => {
        getDeviceId();
    }, []);


    const getDeviceId = async () => {
        try {
            const deviceId = await invoke<string>('get_device_id');
            debug(`📣 Received Device Id: ${deviceId}`);
            setDeviceId(deviceId);
        } catch (err) {
            logError(`Failed to get device id: ${err}`);
            setDeviceId(null);
        }
    };


    return {deviceId};
}