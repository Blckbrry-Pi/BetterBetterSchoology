
use bbs_shared::{PageState, StateUpdateAction, DataUpdateAction, errors::LoginError, FrontendData};
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, Properties, html, use_context, UseReducerHandle, Callback};

use web_sys::{ window, HtmlElement, HtmlInputElement};
use wasm_bindgen::JsCast;

use crate::{build_classes, get_class_listing};

use super::set_credentials_foreign;



#[derive(Debug, Properties, PartialEq, Eq)]
pub struct LoginDataPageProps {
    pub username: String,
    pub password: String,
}

const LOGIN_INPUT: &str = "w-64 h-8 rounded-md bg-slate-600 border-[1px] border-slate-500 p-2 m-1";

#[function_component(LoginPage)]
pub fn login_page(props: &LoginDataPageProps) -> Html {
    let state = use_context::<UseReducerHandle<PageState>>().expect("no state ctx found");
    let data = use_context::<UseReducerHandle<FrontendData>>().expect("no data ctx found");

    let LoginDataPageProps { username, password } = props;
    let sumbit_username = username.clone();
    let sumbit_password = password.clone();
    
    let app_state_for_uname = state.clone();
    let app_state_for_passw = state.clone();

    let app_state_for_entering = state;

    let user_change_event = move |event: yew::events::InputEvent| {
        app_state_for_uname.dispatch(
            StateUpdateAction::SetUname(
                event.target().unwrap().dyn_into::<HtmlInputElement>().unwrap().value()
            )
        )
    };

    let user_enter_event = move |event: yew::events::KeyboardEvent| {
        if event.key() == "Enter" {
            if let Some(el) = (|| window()
                ?.document()
                ?.query_selector("#password-field")
                .ok()
                .flatten()
                ?.dyn_into::<HtmlElement>()
                .ok()
            )() {
                el.focus().unwrap();
            }
        }
    };

    let pass_change_event = move |event: yew::events::InputEvent| {
        app_state_for_passw.dispatch(
            StateUpdateAction::SetPassw(
                event.target().unwrap().dyn_into::<HtmlInputElement>().unwrap().value()
            )
        )
    };

    let pass_enter_event = move |event: yew::events::KeyboardEvent| {

        if event.key() == "Enter" {
            let app_state_for_entering = app_state_for_entering.clone();
            let app_state_for_failing  = app_state_for_entering.clone();

            let app_data = data.clone();

            let username = sumbit_username.clone();
            let password = sumbit_password.clone();

            spawn_local(async move {
                if let Err(e) = set_credentials_foreign(username, password).await {
                    window()
                        .unwrap()
                        .alert_with_message(&format!("Failed to set credentials! {:#?}", e))
                        .unwrap();
                } else {
                    app_state_for_entering.dispatch(StateUpdateAction::LogIn);

                    let _ = get_class_listing(
                        Callback::from(move |new_data| {
                            app_state_for_entering.dispatch(StateUpdateAction::ToMain);
                            app_data.dispatch(DataUpdateAction::SetClassListing(new_data));
                        }),
                        Callback::from(move |error| app_state_for_failing.dispatch(StateUpdateAction::FailLogin(error))),
                    );
                }

            });
        }
        
    };


    html! {
        <div class={"flex flex-col items-center justify-center h-full"}>
            <h1 class={"text-4xl font-sans text-center mb-3"}>
                {"Log in to"}<br/>
                <span class={"text-3xl border-2 border-white rounded-full w-10 h-10 inline-block align-bottom leading-9 mr-2"}>
                    {"S"}
                </span>
                {"schoolo"}<span class={"text-2xlplus font-semibold"}>{"GY"}<sup class={"font-normal"}>{"®"}</sup></span>
            </h1>
            <input
                class={LOGIN_INPUT}
                oninput={user_change_event}
                onkeyup={user_enter_event}
                id={"username-field"}
                placeholder={"Username"}
                type={"email"}
                value={ username.clone() } />
            <input
                class={LOGIN_INPUT}
                oninput={pass_change_event}
                onkeyup={pass_enter_event}
                id={"password-field"}
                placeholder={"Password"}
                type={"password"}
                value={ password.clone() } />
        </div>
    }
}



#[derive(Debug, Properties, PartialEq)]
pub struct LoginOverlayProps {
    pub loading: bool,
    pub error: Option<LoginError>,
    pub return_to_login: Option<Callback<()>>,
}


const FLEX_COLUMN_CENTER: &str = "flex flex-col items-center justify-center";

const ERROR_BASE: &str = build_classes!(
    "h-screen w-screen fixed top-0",
    "bg-slate-200",
    FLEX_COLUMN_CENTER,
    "transition-[background-opacity,opacity,background-color,filter,backdrop-filter,-webkit-backdrop-filter] duration-500 ease-in-out",
);
const SHOW_ERROR: &str = build_classes!(
    ERROR_BASE,
    "bg-opacity-5 backdrop-blur-md z-50",
);
const HIDE_ERROR: &str = build_classes!(
    ERROR_BASE,
    "bg-opacity-0 backdrop-blur-none pointer-events-none",
);

const INNER_BASE: &str = build_classes!(
    FLEX_COLUMN_CENTER,
    "transition-opacity duration-300 ease-in-out",
);
const SHOW_INNER: &str = build_classes!(
    INNER_BASE,
    "opacity-100",
);
const HIDE_INNER: &str = build_classes!(
    INNER_BASE,
    "opacity-0",
);


#[function_component(LoginOverlay)]
pub fn login_overlay(props: &LoginOverlayProps) -> Html {
    use LoginError::*;
    let message = match props.error {
        Some(SerializationError) => html! {
            <h1 class="text-2xl">{"Internal error encountered."}</h1>
        },
        Some(FindFormError) => unreachable!("Find form error should be propogated to logged in."),
        Some(InvalidCredsError) => html! {
            <h1>{"Invalid username or password."}</h1>
        },
        Some(RequestError) => html! {
            <>
                <h1 class=" text-2xl">{"Failed to reach server."}</h1>
                <h3>{"Try checking your internet connection."}</h3>
            </>
        },
        Some(LaterRequestError) => html! {
            <>
                <h1>{"Transiently failed to fulfill request."}</h1>
                <h3>{"This may be a problem with your internet or with the app."}</h3>
                <h3>{"Please alert the developer if this issue persists."}</h3>
            </>
        },
        Some(DecodeError) => html! {
            <>
                <h1>{"Invalid response text."}</h1>
                <h3>{"This is likely an issue with Schoology."}</h3>
                <h3>{"Please alert the developer if this issue persists."}</h3>
            </>
        },
        Some(JsonError) => html! {
            <>
                <h1>{"Invalid JSON detected."}</h1>
                <h3>{"This could either be a issue with schoology or with the app."}</h3>
                <h3>{"Please alert the developer to this bug as soon as possible."}</h3>
            </>
        },
        None => html! {},
    };

    html! {
        <div class={if props.error.is_some() || props.loading { SHOW_ERROR } else { HIDE_ERROR }} >
            <div class={if props.error.is_some() { SHOW_INNER } else { HIDE_INNER } }>
                {message}
                {
                    if let Some(callback) = props.return_to_login.clone() {
                        html! {<button onclick={move |_| callback.emit(())} class="mt-3 px-3 py-2 bg-violet-400 rounded-md text-black">{"<-- Return to Login"}</button>}
                    } else {
                        html! {}
                    }
                }
            </div>
        </div>
    }
}