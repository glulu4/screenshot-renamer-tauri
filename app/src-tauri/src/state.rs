use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub paused: Arc<Mutex<bool>>,
}
