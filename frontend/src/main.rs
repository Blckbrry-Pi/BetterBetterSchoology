use std::ops::Deref;

use bbs_shared::StateUpdateAction;
use bbs_shared::data::ClassEntry;
use bbs_shared::errors::LoginError;
use bbs_shared::{ PageState, FrontendData, DataUpdateAction };

use frontend::{MainPage};
use frontend::{LoginPage, LoginOverlay};
use frontend::{Breadcrumb, Breadcrumbs};

use bincode::deserialize;
use base64::decode;

use frontend::{get_class_listing_foreign, parse_single_class_info, reducer_contexts};

use wasm_bindgen::JsValue;
use web_sys::console::log_2;
use yew::prelude::*;

use wasm_bindgen_futures::spawn_local;

use web_sys::{window, console};

static DAY_NAMES: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

fn main() {
    console_error_panic_hook::set_once();
    yew::start_app::<App>();
}

#[function_component(App)]
pub fn app() -> Html {
    spawn_local(async { parse_single_class_info("5202064601".into()).await.unwrap(); });

    let app_state = use_reducer_eq(|| PageState::Login {
        username: String::new(),
        password: String::new(),
    });

    let app_data = use_reducer_eq(FrontendData::empty);

    let home_callback_app_state = app_state.clone();

    let home_callback: Callback<()> = (move |_| home_callback_app_state.dispatch(StateUpdateAction::ToMain)).into();

    use PageState::*;

    let inner = match app_state.deref() {
        Login {
            username,
            password,
        } => {
            html! { <>
                <LoginOverlay loading=false key=0u8/>
                <LoginPage username={username.clone()} password={password.clone()} />
            </> }
        },

        LoggingIn {
            username,
            password,
        } => {
            html! { <>
                <LoginOverlay loading=true key=0u8/>
                <LoginPage username={username.clone()} password={password.clone()} />
            </> }
        }

        LoginFailed {
            username,
            password,
            reason,
        } => {
            let return_app_state = app_state.clone();
            log_2(&JsValue::from_str("Login Failed!"), &JsValue::from_str(&format!("{:?}", reason)));
            html! { <>
                <LoginOverlay error={*reason} return_to_login={Callback::from(move |_| return_app_state.dispatch(StateUpdateAction::ReturnLogin))} loading=false key=0u8/>
                <LoginPage username={username.clone()} password={password.clone()} />
            </> }
        },

        Main { day } => {
            let breadcrumbs = if let Some(day) = day {
                html! {
                    <Breadcrumbs>
                        <Breadcrumb text="Home" on_click_callback={home_callback} has_next=true key=0u8/>
                        <Breadcrumb text={DAY_NAMES[*day].to_string()} on_click_callback={Callback::<()>::from(|_| ())} key=1u8/>
                    </Breadcrumbs>
                }
            } else {
                html! {
                    <Breadcrumbs>
                        <Breadcrumb text="Home" on_click_callback={home_callback} key=0u8/>
                    </Breadcrumbs>
                }
            };
            console::log_1(&JsValue::from_str(&format!("{:?}", app_data.classes)));
            console::log_1(&JsValue::from_bool(app_data.classes == FrontendData::empty().classes));
            html! { <>
                <LoginOverlay loading=false key=0u8/>
                <div>
                    {breadcrumbs}
                    <MainPage day={*day} classes={app_data.classes.clone()} />
                    <LoginOverlay loading=false/>
                </div>
            </>}
        },
        ClassPage {
            id,
            expanded_folders: _,
        } => html! {
            <div>
                <h2>{"Class page!"}</h2>
                <p class="id">{id.0.to_string()}</p>
            </div>
        },
        ClassItemPage {
            id,
            page_specific_data: _,
        } => html! {
            <div>
                <h2>{"Class page!"}</h2>
                <p class="id">{id.0.to_string()}</p>
            </div>
        },
    };

    // html! {
    //     <ReducCtx<PageState> context={app_state.clone()}>
    //         <ReducCtx<FrontendData> context={app_data.clone()}>
    //             <div class={"h-screen bg-slate-800 text-white overflow-scroll pl-7"}>
    //                 {inner}
    //             </div>
    //         </ReducCtx<FrontendData>>
    //     </ReducCtx<PageState>>
    // }
    
    reducer_contexts! { PageState: app_state, FrontendData: app_data =>
        <div class={"h-screen bg-slate-800 text-white overflow-scroll"}>
            {inner}
        </div>
    }
}


