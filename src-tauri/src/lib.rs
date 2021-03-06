pub mod commands;
pub mod requests;
pub mod structs;

use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    username: Mutex<Arc<String>>,
    password: Mutex<Arc<String>>,
}

impl Default for Credentials {
    fn default() -> Self {
        Self {
            username: Mutex::new(Default::default()),
            password: Mutex::new(Default::default()),
        }
    }
}


unsafe impl Sync for Credentials {}


