use serde::{Serialize, Deserialize};

use crate::{ClassID, ClassItemID, errors::LoginError};

#[derive(Clone, Debug,Serialize, Deserialize, PartialEq, Eq)]
pub enum PageState {
    Login {
        username: String,
        password: String,
    },
    LoggingIn {
        username: String,
        password: String,
    },
    LoginFailed {
        username: String,
        password: String,
        reason: LoginError,
    },
    Main {
        day: Option<usize>
    },
    ClassPage {
        id: ClassID,
        expanded_folders: Vec<ClassItemID>,
    },
    ClassItemPage {
        id: ClassItemID,
        page_specific_data: (),
    },
}

impl PageState {
    pub fn is_login(&self) -> bool {
        if let PageState::Login { .. } | PageState::LoginFailed { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn as_login_username(&self) -> Option<&String> {
        if let PageState::Login { username, .. } | PageState::LoginFailed { username, .. } = self {
            Some(username)
        } else {
            None
        }
    }
    pub fn as_login_password(&self) -> Option<&String> {
        if let PageState::Login { password, .. } | PageState::LoginFailed { password, .. } = self {
            Some(password)
        } else {
            None
        }
    }
    pub fn is_main(&self) -> bool {
        if let PageState::Main { .. } = self {
            true
        } else {
            false
        }
    }
}
