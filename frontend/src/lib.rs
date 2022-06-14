mod login;
mod breadcrumbs;
mod main_page;
mod class_page;

use base64::decode;
use bbs_shared::{data::ClassEntry, errors::LoginError, ClassID, FrontendData, PageState, StateUpdateAction, DataUpdateAction, SectionID};
use bincode::deserialize;

pub use login::{ LoginPage, LoginOverlay, LoginOverlayProps };
pub use main_page::MainPage;
pub use class_page::{ClassPage, ClassPageOverlay, ClassPageOverlayProps};
pub use breadcrumbs::{ Breadcrumbs, Breadcrumb, BreadcrumbProps };


use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, console};
use yew::{Callback, UseReducerHandle};

#[macro_export]
macro_rules! reducer_contexts {
    (() => $($tail:tt)*) => {{
        html!{
            $($tail)*
        }
    }};
    ($typ:tt: $val:expr => $($tail:tt)*) => {{
        html!{
            <ContextProvider<UseReducerHandle<$typ>> context={($val).clone()}>
                $($tail)*
            </ContextProvider<UseReducerHandle<$typ>>>
        }
    }};
    ($typ:tt: $val:expr, $($typ_list:tt: $val_list:expr),* => $($tail:tt)*) => {{
        html!{
            <ContextProvider<UseReducerHandle<$typ>> context={($val).clone()}>
                {reducer_contexts! { $($typ_list: $val_list),* => $($tail)* }}
            </ContextProvider<UseReducerHandle<$typ>>>
        }
    }};
}

#[wasm_bindgen(module = "/public/glue.js")]
extern "C" {
    #[wasm_bindgen(js_name = invokeIsLoggedIn, catch)]
    pub async fn is_logged_in_foreign() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = invokeSetCredentials, catch)]
    pub async fn set_credentials_foreign(username: String, password: String) -> Result<(), JsValue>;

    #[wasm_bindgen(js_name = invokeGetClassListing, catch)]
    pub async fn get_class_listing_foreign() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = parseSingleClassInfo, catch)]
    pub async fn parse_single_class_info(classid: String) -> Result<JsValue, JsValue>;
}
pub fn get_class_listing(data_callback: Callback<Vec<ClassEntry>>, error_callback: Callback<LoginError>) {
    async fn get_class_listing_guts(data_callback: Callback<Vec<ClassEntry>>, error_callback: Callback<LoginError>) {
        let opt_str = match get_class_listing_foreign().await {
            Ok(val) => val.as_string(),
            Err(err) => {
                match err.as_string().map(TryFrom::try_from) {
                    Some(Ok(err)) => error_callback.emit(err),
                    Some(Err(de_err)) => window()
                        .unwrap()
                        .alert_with_message(&format!("Failed to deserialize recieved error: {:?}", de_err))
                        .unwrap(),
                    None => window()
                        .unwrap()
                        .alert_with_message(&format!("Returned error was not a string: {:?}", err))
                        .unwrap(),
                }
                return;
            }
        };

        let data_str = match opt_str {
            Some(s) => s,
            None => panic!(),
        };

        let data_buf = match decode(&data_str) {
            Ok(buf) => buf,
            Err(err) => {
                window()
                    .unwrap()
                    .alert_with_message(&format!("Text: {}\nError: {:?}", &data_str, err))
                    .unwrap();
                return;
            }
        };

        match deserialize(&data_buf) {
            Ok(des_state) => {
                data_callback.emit(des_state);
            }
            Err(err) => {
                window()
                    .unwrap()
                    .alert_with_message(&format!("Text: {}\nError: {:?}", &data_str, err))
                    .unwrap();
            }
        }
    }

    spawn_local({
        get_class_listing_guts(data_callback, error_callback)
    });
}

pub async fn is_logged_in() -> bool {
    let string = match is_logged_in_foreign().await {
        Ok(js_val) => match js_val.as_string() {
            Some(string) => string,
            None => return false,
        },
        Err(_) => return false,
    };

    let buffer = match decode(&string) {
        Ok(buffer) => buffer,
        Err(_) => return false,
    };

    deserialize(&buffer).unwrap_or(false)
}

pub fn dispatch_load_class(
    ids: (ClassID, SectionID),
    state_handle: UseReducerHandle<PageState>,
    data_handle: UseReducerHandle<FrontendData>,
) {
    async fn dispatch_load_class_inner(
        (id, section_id): (ClassID, SectionID),
        state_handle: UseReducerHandle<PageState>,
        data_handle: UseReducerHandle<FrontendData>,
    ) {
        console::log_1(&id.0.to_string().as_str().into());
        let data = match parse_single_class_info(section_id.0.to_string()).await {
            Ok(data) => {
                if let Some(data) = data.as_string() {
                    data
                } else {
                    console::error_2(&"step 2".into(), &data);
                    state_handle.dispatch(StateUpdateAction::ToMain);
                    return;
                }
            },
            Err(err) => {
                console::error_2(&"step 1".into(), &err);
                state_handle.dispatch(StateUpdateAction::ToMain);
                return;
            }
        };
        let data = match decode(data) {
            Ok(data) => data,
            Err(err) => {
                console::error_2(&"step 3".into(), &err.to_string().into());
                state_handle.dispatch(StateUpdateAction::ToMain);
                return;
            }
        };

        match deserialize::<Vec<Vec<_>>>(&data) {
            Ok(materials_data) => data_handle.dispatch(DataUpdateAction::SetClassPageInfo(materials_data.into_iter().flatten().collect())),
            Err(err) => {
                console::error_2(&"step 4".into(), &err.to_string().into());
                state_handle.dispatch(StateUpdateAction::ToMain);
                return;
            }
        }
        state_handle.dispatch(StateUpdateAction::ToClass(id));
    }
    spawn_local(dispatch_load_class_inner(ids, state_handle, data_handle));
}