import {useState, useEffect} from 'react';
import {UserDevice} from '../types';
import {doc, getFirestore, onSnapshot} from 'firebase/firestore';
import {app} from '../../firebaseConfig'
import {debug, info, error as logError} from '@tauri-apps/plugin-log';
import {invoke} from '@tauri-apps/api/core';
// import {User} from '@/types/user';
// import {handleError} from '@/utils/util';

export const useLiveUser = () => {


    const [deviceId, setDeviceId] = useState<string | null>(null);

    const getDeviceId = async () => {
        try {
            const deviceId = await invoke<string>('get_device_id');
            debug(`📣 Received Device Id in : ${deviceId}`);
            setDeviceId(deviceId);
        } catch (err) {
            logError(`Failed to get device id: ${err}`);
            setDeviceId(null);
        }
    };

    useEffect(() => {
        getDeviceId();
    }, []);

    const [userDevice, setUserDevice] = useState<UserDevice | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<Error | null>(null);

    useEffect(() => {


        try {
            const db = getFirestore(app);

            info(`🔄 Subscribing to live user data for device ID: ${deviceId}`);
            if (!deviceId) {
                setLoading(false);
                setError(new Error('User ID is required'));
                return;
            }

            const unsubscribe = onSnapshot(
                doc(db, 'UserDevice', deviceId),
                (docSnap) => {
                    if (docSnap.exists()) {
                        debug(`📣 Live user data updated: ${JSON.stringify(docSnap.data())}`);
                        setUserDevice(docSnap.data() as UserDevice);
                    }
                    else{
                        debug(`📣 No user data found for device ID: ${deviceId}`);
                        setUserDevice(null);
                        setError(new Error('No user data found'));
                    }
                    setLoading(false);
                },
                (err) => {

                    setError(err);
                    setLoading(false);
                }
            );

            return () => unsubscribe();
        } catch (error) {
            logError(`Error subscribing to live user data: ${error}`);
            setError(error as Error);
            setLoading(false);
        }
    }, [deviceId]);

    return {userDevice, loading, error, deviceId};
};
