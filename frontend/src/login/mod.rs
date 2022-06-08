
use bbs_shared::{PageState, StateUpdateAction};
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, Properties, html, use_context, UseReducerHandle};

use web_sys::{ window, HtmlElement, HtmlInputElement};
use wasm_bindgen::JsCast;

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
            let app_state = app_state_for_entering.clone();

            let username = sumbit_username.clone();
            let password = sumbit_password.clone();

            spawn_local(async move {
                if let Err(e) = set_credentials_foreign(username, password).await {
                    window()
                        .unwrap()
                        .alert_with_message(&format!("Failed to set credentials! {:#?}", e))
                        .unwrap();
                } else {
                    app_state.dispatch(StateUpdateAction::ToMain)
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
