use std::{path::PathBuf, sync::{Arc, Mutex}};

use crate::user::UserDevice;



#[derive(Clone)]
pub struct AppState {
    pub paused: Arc<Mutex<bool>>,
    pub user_device: UserDevice,
    pub selected_path: Arc<Mutex<String>>,
}
