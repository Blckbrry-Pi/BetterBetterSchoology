use std::ops::Deref;

use bbs_shared::{StateUpdateAction, DataUpdateAction};
use bbs_shared::{ PageState, FrontendData };

use frontend::MainPage;
use frontend::{LoginPage, LoginOverlay, LoginOverlayProps};
use frontend::{ClassPageOverlayProps, ClassPageOverlay};
use frontend::{BreadcrumbProps, Breadcrumbs};

use frontend::{is_logged_in, get_class_listing, parse_single_class_info, reducer_contexts};

use wasm_bindgen::JsValue;
use yew::{prelude::*, props};

use wasm_bindgen_futures::spawn_local;

use web_sys::console;

static DAY_NAMES: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

fn main() {
    console_error_panic_hook::set_once();
    yew::start_app::<App>();
}

#[function_component(App)]
pub fn app() -> Html {
    // spawn_local(async { parse_single_class_info("5202064601".into()).await.unwrap(); });

    let app_state = use_reducer_eq(|| PageState::LoggingIn {
        username: String::new(),
        password: String::new(),
    });

    let app_data = use_reducer_eq(FrontendData::empty);

    let callback_app_state = app_state.clone();
    let callback_app_data = app_data.clone();
    use_effect_with_deps(move |_| {
        spawn_local(async move {
            if is_logged_in().await {
                let success_callback_app_state = callback_app_state.clone();
                let failure_callback_app_state = callback_app_state;

                let _ = get_class_listing(
                    Callback::from(move |new_data| {
                        success_callback_app_state.dispatch(StateUpdateAction::ToMain);
                        callback_app_data.dispatch(DataUpdateAction::SetClassListing(new_data));
                    }),
                    Callback::from(move |error| failure_callback_app_state.dispatch(StateUpdateAction::FailLogin(error))),
                );

            } else {
                callback_app_state.dispatch(StateUpdateAction::ToLogin);
            }
        });
        || ()
    }, ());

    let home_callback_app_state = app_state.clone();

    let home_callback: Callback<()> = (move |_| home_callback_app_state.dispatch(StateUpdateAction::ToMain)).into();

    console::log_1(&JsValue::from_str(&format!("{:?}", app_state)));

    use PageState::*;

    let login_overlay_props;
    let breadcrumbs: Option<Vec<BreadcrumbProps>>;
    let class_page_overlay_props;

    let inner = match app_state.deref() {
        Login {
            username,
            password,
        } => {
            login_overlay_props = LoginOverlayProps {
                loading: false,
                error: None,
                return_to_login: None,
            };
            breadcrumbs = None;
            class_page_overlay_props = ClassPageOverlayProps {
                loading: false,
                error: None,
                return_to_login: None,
            };
            html! { <>
                <LoginPage username={username.clone()} password={password.clone()} />
            </> }
        },

        LoggingIn {
            username,
            password,
        } => {
            login_overlay_props = LoginOverlayProps {
                loading: true,
                error: None,
                return_to_login: None,
            };
            breadcrumbs = None;
            class_page_overlay_props = ClassPageOverlayProps {
                loading: false,
                error: None,
                return_to_login: None,
            };
            html! { <>
                <LoginPage username={username.clone()} password={password.clone()} />
            </> }
        }

        LoginFailed {
            username,
            password,
            reason,
        } => {
            let return_app_state = app_state.clone();
            login_overlay_props = LoginOverlayProps {
                loading: false,
                error: Some(*reason),
                return_to_login: Some(Callback::from(move |_| return_app_state.dispatch(StateUpdateAction::ReturnLogin))),
            };
            breadcrumbs = None;
            class_page_overlay_props = ClassPageOverlayProps {
                loading: false,
                error: None,
                return_to_login: None,
            };
            html! { <>
                <LoginPage username={username.clone()} password={password.clone()} />
            </> }
        },

        Main { day } => {
            login_overlay_props = LoginOverlayProps {
                loading: false,
                error: None,
                return_to_login: None,
            };
            breadcrumbs = if let Some(day) = day {
                Some(vec![
                    props!(BreadcrumbProps {
                        text: "Home",
                        on_click_callback: home_callback,
                    }),
                    props!(BreadcrumbProps {
                        text: DAY_NAMES[*day].to_string(),
                        on_click_callback: Callback::<()>::from(|_| ()),
                        has_next: false,
                    }),
                ])
            } else {
                Some(vec![
                    props!(BreadcrumbProps {
                        text: "Home",
                        on_click_callback: Callback::<()>::from(|_| ()),
                        has_next: false,
                    }),
                ])
            };
            class_page_overlay_props = ClassPageOverlayProps {
                loading: false,
                error: None,
                return_to_login: None,
            };
            html! { <>  
                <div>
                    <MainPage day={*day} classes={app_data.classes.clone()} key=0u8/>
                </div>
            </>}
        },
        LoadingClass {
            class_id,
        } => {
            let class_name = app_data
                .classes
                .borrow()
                .as_ref()
                .map(
                    |classes| classes
                        .iter()
                        .find(|entry| &entry.id == class_id)
                        .map(|entry| entry.name.clone())
                )
                .flatten()
                .unwrap_or_else(|| class_id.0.to_string());

            login_overlay_props = LoginOverlayProps {
                loading: false,
                error: None,
                return_to_login: None,
            };
            breadcrumbs = Some(vec![
                props!(BreadcrumbProps {
                    text: "Home",
                    on_click_callback: home_callback,
                }),
                props!(BreadcrumbProps {
                    text: class_name,
                    on_click_callback: Callback::<()>::from(|_| ()),
                    has_next: false,
                    unbounded: true,
                }),
            ]);
            class_page_overlay_props = ClassPageOverlayProps {
                loading: true,
                error: None,
                return_to_login: None,
            };
            html! {
                <MainPage day={None} classes={app_data.classes.clone()} key=0u8/>
            }
        },
        ClassPage {
            id,
            expanded_folders: _,
        } => {
            let class_name = app_data
                .classes
                .borrow()
                .as_ref()
                .map(
                    |classes| classes
                        .iter()
                        .find(|entry| &entry.id == id)
                        .map(|entry| entry.name.clone())
                )
                .flatten()
                .unwrap_or_else(|| id.0.to_string());

            login_overlay_props = LoginOverlayProps {
                loading: false,
                error: None,
                return_to_login: None,
            };
            breadcrumbs = Some(vec![
                props!(BreadcrumbProps {
                    text: "Home",
                    on_click_callback: home_callback,
                }),
                props!(BreadcrumbProps {
                    text: class_name,
                    on_click_callback: Callback::<()>::from(|_| ()),
                    has_next: false,
                    unbounded: true,
                }),
            ]);
            class_page_overlay_props = ClassPageOverlayProps {
                loading: false,
                error: None,
                return_to_login: None,
            };
            use frontend::ClassPage;
            html! {
                <div>
                    <ClassPage
                        materials={app_data.curr_class_data.clone()}/>
                </div>
            }
        },
        ClassItemPage {
            id,
            page_specific_data: _,
        } => {
            login_overlay_props = LoginOverlayProps {
                loading: false,
                error: None,
                return_to_login: None,
            };
            class_page_overlay_props = ClassPageOverlayProps {
                loading: false,
                error: None,
                return_to_login: None,
            };
            breadcrumbs = Some(vec![
                props!(BreadcrumbProps {
                    text: "Home",
                    on_click_callback: home_callback,
                }),
                props!(BreadcrumbProps {
                    text: id.0.to_string(),
                    on_click_callback: Callback::<()>::from(|_| ()),
                }),
                props!(BreadcrumbProps {
                    text: id.0.to_string(),
                    on_click_callback: Callback::<()>::from(|_| ()),
                    has_next: false,
                }),
            ]);
            html! {
                <div>
                    <h2>{"Class page!"}</h2>
                    <p class="id">{id.0.to_string()}</p>
                </div>
            }
        },
    };
    
    reducer_contexts! { PageState: app_state, FrontendData: app_data =>
        <div class={"h-screen bg-slate-800 text-white overflow-scroll"}>
            <LoginOverlay ..login_overlay_props/>
            <ClassPageOverlay ..class_page_overlay_props/>
            {if let Some(breadcrumbs) = breadcrumbs { html! {<Breadcrumbs children={breadcrumbs}/>} } else {  html! {} }}
            {inner}
        </div>
    }
}


