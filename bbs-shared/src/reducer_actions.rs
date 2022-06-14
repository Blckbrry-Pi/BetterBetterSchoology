use std::rc::Rc;

use yew::Reducible;

use crate::data::{ClassEntry, Assignment};
use crate::errors::LoginError;
use crate::{ ClassID, MaterialID };
use crate::PageState;
use crate::FrontendData;

#[derive(Debug, Clone)]
pub enum StateUpdateAction {
    ToLogin,
    FailLogin(LoginError),
    LogIn,
    ReturnLogin,
    SetUname(String),
    SetPassw(String),
    ToMain,
    SetDayFilter(usize),
    LoadClass(ClassID),
    ToClass(ClassID),
    ToClassItem(MaterialID)
}

use StateUpdateAction::*;

impl Reducible for PageState {
    type Action = StateUpdateAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ToLogin => Rc::new(PageState::Login {
                username: String::new(),
                password: String::new(),
            }),
            FailLogin(reason) => Rc::new(PageState::LoginFailed {
                username: self.as_login_username().cloned().unwrap_or_default(),
                password: self.as_login_password().cloned().unwrap_or_default(),
                reason,
            }),
            LogIn => Rc::new(PageState::LoggingIn {
                username: self.as_login_username().cloned().unwrap_or_default(),
                password: self.as_login_password().cloned().unwrap_or_default(),
            }),
            ReturnLogin => Rc::new(PageState::Login {
                username: self.as_login_username().cloned().unwrap_or_default(),
                password: self.as_login_password().cloned().unwrap_or_default(),
            }),
            SetUname(username) => Rc::new(PageState::Login {
                username,
                password: self.as_login_password().cloned().unwrap_or_default(),
            }),
            SetPassw(password) => Rc::new(PageState::Login {
                username: self.as_login_username().cloned().unwrap_or_default(),
                password,
            }),
            ToMain => Rc::new(PageState::Main {
                day: None
            }),
            SetDayFilter(day) => Rc::new(PageState::Main {
                day: Some(day)
            }),
            LoadClass(class_id) => Rc::new(PageState::LoadingClass {
                class_id,
            }),
            ToClass(class_id) => Rc::new(PageState::ClassPage {
                id: class_id,
                expanded_folders: vec![],
            }),
            ToClassItem(class_item_id) => Rc::new(PageState::ClassItemPage {
                id: class_item_id,
                page_specific_data: (),
            }),
        }
    }
}


pub enum DataUpdateAction {
    ClearClassListing,
    SetClassListing(Vec<ClassEntry>),
    ClearClassPageInfo,
    SetClassPageInfo(Vec<Assignment>),
}

use DataUpdateAction::*;

impl Reducible for FrontendData {
    type Action = DataUpdateAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ClearClassListing => {
                *self.classes.borrow_mut() = None;
                Rc::new(FrontendData {
                    classes: self.classes.new_inc_clone(),
                    curr_class_data: self.curr_class_data.clone(),
                })
            },
            SetClassListing(class_entries) => {
                *self.classes.borrow_mut() = Some(class_entries);
                Rc::new(FrontendData {
                    classes: self.classes.new_inc_clone(),
                    curr_class_data: self.curr_class_data.clone(),
                })
            },
            ClearClassPageInfo => {
                *self.curr_class_data.borrow_mut() = None;
                Rc::new(FrontendData {
                    classes: self.classes.clone(),
                    curr_class_data: self.curr_class_data.new_inc_clone(),
                })
            },
            SetClassPageInfo(class_page_data) => {
                *self.curr_class_data.borrow_mut() = Some(class_page_data);
                Rc::new(FrontendData {
                    classes: self.classes.clone(),
                    curr_class_data: self.curr_class_data.new_inc_clone(),
                })
            },
        }
    }
}