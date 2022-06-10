mod login;
mod breadcrumbs;
mod main_page;

use base64::decode;
use bbs_shared::{data::ClassEntry, errors::LoginError};
use bincode::deserialize;
pub use main_page::MainPage;

pub use login::{ LoginPage, LoginOverlay };
pub use breadcrumbs::{ Breadcrumbs, Breadcrumb };


use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::Callback;

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
    #[wasm_bindgen(js_name = invokeGetClassListing, catch)]
    pub async fn get_class_listing_foreign() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = invokeSetCredentials, catch)]
    pub async fn set_credentials_foreign(username: String, password: String) -> Result<(), JsValue>;
    #[wasm_bindgen(js_name = parseSingleClassInfo, catch)]
    pub async fn parse_single_class_info(classid: String) -> Result<JsValue, JsValue>;
}
fn get_class_listing(data_callback: Callback<Vec<ClassEntry>>, error_callback: Callback<LoginError>) -> impl FnOnce() {
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
    || ()
}