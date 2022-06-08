mod login;
mod breadcrumbs;
mod main_page;

pub use main_page::MainPage;

pub use login::LoginPage;
pub use breadcrumbs::{ Breadcrumbs, Breadcrumb };


use wasm_bindgen::prelude::*;

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
    
}
