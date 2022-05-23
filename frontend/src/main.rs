use std::ops::Deref;

use bbs_shared::{ PageState, FrontendData, DataUpdateAction, data::SectionDataGuts };
use frontend::MainPageClass;
use frontend::LoginPage;

use bincode::deserialize;
use base64::decode;

use frontend::get_class_listing_foreign;
use yew::prelude::*;

use wasm_bindgen_futures::spawn_local;

use web_sys::window;

fn main() {
    yew::start_app::<App>();
}


#[function_component(App)]
pub fn app() -> Html {
    let app_state = use_reducer_eq(|| PageState::Login {
        username: String::new(),
        password: String::new(),
    });
    let app_data = use_reducer(|| FrontendData::empty());

    {
        if app_state.is_main() {
            let app_data = app_data.clone();
            use_effect_with_deps(
                move |_| get_class_listing(app_data),
                ()
            );
        }
    }

    use PageState::{ Main, Login, LoginFailed, ClassPage, ClassItemPage };

    let inner = match app_state.deref() {
        Login {
            username,
            password,
        } => {
            html! { <LoginPage username={username.clone()} password={password.clone()} /> }
        },

        LoginFailed { .. } => {
            html! { }
        },

        Main { day } => {
            let classes_ref =  app_data.classes.borrow();
            let class_html = match classes_ref.as_ref() {
                Some(a) => html! {
                    {
                        a
                            .iter()
                            .map(|entry| (
                                entry,
                                if let Some(day) = day {
                                    if let SectionDataGuts::Good { days, .. } = entry.section.guts {
                                        days[*day]
                                    } else {false}
                                } else {true}
                            ))
                            .map(|(entry, enabled)| html! {
                                <MainPageClass entry={entry.clone()} enabled={enabled} key={entry.id.0}/>
                            })
                            .collect::<Html>()
                    }
                },
                None => html! {
                    <h1>{"Loading..."}</h1>
                },
            };
            html! {
                <div>
                    <h2>{"Home:"}</h2>
                    {class_html}
                </div>
            }
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

    html! {
        <ContextProvider<UseReducerHandle<PageState>> context={app_state.clone()}>
            <div class={"h-screen bg-slate-800 text-white overflow-scroll pl-7"}>
                 {inner}
            </div>
        </ContextProvider<UseReducerHandle<PageState>>>
    }

    
}

fn get_class_listing(data_handle: UseReducerHandle<FrontendData>) -> impl FnOnce() -> () {
    async fn get_class_listing_guts(data_handle: UseReducerHandle<FrontendData>) {
        let opt_str = match get_class_listing_foreign().await {
            Ok(val) => val.as_string(),
            Err(err) => {
                window()
                    .unwrap()
                    .alert_with_message(&format!("Recieved value error encountered: {:?}", err.as_string()))
                    .unwrap();
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
                let window = window().unwrap();
                window
                    .alert_with_message(&format!("Text: {}\nError: {:?}", &data_str, err))
                    .unwrap();
                return;
            }
        };

        match deserialize(&data_buf) {
            Ok(des_state) => {
                data_handle.dispatch(DataUpdateAction::SetClassListing(des_state));
            }
            Err(err) => {
                let window = window().unwrap();
                window
                    .alert_with_message(&format!("Text: {}\nError: {:?}", &data_str, err))
                    .unwrap();
            }
        }
    }

    spawn_local({
        get_class_listing_guts(data_handle)
    });
    || ()
}
