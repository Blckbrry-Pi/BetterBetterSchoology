use crate::build_classes;

use yew::{function_component, Properties, html, Callback, Children, use_state_eq};

#[derive(Debug, Properties, PartialEq)]
pub struct BreadcrumbProps {
    pub text: Option<String>,
    pub on_click_callback: Callback<()>,
    #[prop_or(false)]
    pub has_next: bool,
    #[prop_or(false)]
    pub hidden: bool,
}

const ARROW_BASE_CLASSES: &str = "h-3 w-3 border-r-2 border-t-2 rotate-45 transition-all duration-300";

const ARROW_NO_NEXT_CLASSES: &str = build_classes!(
    ARROW_BASE_CLASSES,
    "ml-[-0.75rem] scale-x-0, opacity-0",
);

const ARROW_HAS_NEXT_CLASSES: &str = build_classes!(
    ARROW_BASE_CLASSES,
    "ml-2 scale-x-100 opacity-100",
);

const CRUMB_BASE_CLASSES: &str = "h-12 w-24 inline-flex items-center justify-center relative transition-all duration-300";

const CRUMB_HIDDEN_CLASSES: &str = build_classes!(
    CRUMB_BASE_CLASSES,
    "opacity-0 left-[-0.75rem]",
);

const CRUMB_SHOWING_CLASSES: &str = build_classes!(
    CRUMB_BASE_CLASSES,
    "opacity-100 left-0"
);

#[function_component(Breadcrumb)]
pub fn breadcrumb(props: &BreadcrumbProps) -> Html {
    let text = use_state_eq(|| "Placeholder".to_string());

    {
        if let Some(prop_text) = &props.text {
            if !text.as_str().eq(prop_text) {
                text.set(prop_text.to_string());
            }
        }
    }

    let owned_callback = props.on_click_callback.clone();
    html! {
        <div
            class={if props.hidden { CRUMB_HIDDEN_CLASSES } else { CRUMB_SHOWING_CLASSES } }
            onclick={move |_| owned_callback.emit(())} >
            <span class="text-lg">{text.as_str()}</span>
            <div class={if props.has_next { ARROW_HAS_NEXT_CLASSES } else { ARROW_NO_NEXT_CLASSES }}/>
        </div>
    }
}

#[derive(Debug, Properties, PartialEq)]
pub struct BreadcrumbsProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Breadcrumbs)]
pub fn breadcrumbs(props: &BreadcrumbsProps) -> Html {
    html! {
        <div>
            {props.children.iter().chain([html! { <Breadcrumb on_click_callback={Callback::<()>::from(|_| ())} hidden=true key={props.children.len()}/> }].into_iter()).collect::<Vec<_>>()}
        </div>
    }
}
