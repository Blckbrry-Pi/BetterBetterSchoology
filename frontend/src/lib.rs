pub mod login;
pub mod breadcrumbs;

use bbs_shared::{data::{ClassEntry, SectionData, SectionDataGuts}, PageState, StateUpdateAction};
use yew::{function_component, Properties, html, Html, use_context, UseReducerHandle};
use wasm_bindgen::{prelude::*};

pub use {login::LoginPage, breadcrumbs::{Breadcrumb, Breadcrumbs}};

#[macro_export]
macro_rules! build_classes {
    ()=>{""};
    ($($arg: expr),* $(,)?)=>(
        {
            const_format::concatcp!(
                $( ( $arg ), " ",)*
            )
        } as &'static str
    );
}

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct MainPageClassProps {
    pub entry: ClassEntry,
    pub enabled: bool,
}

const BOX_BASE: &str = build_classes!(
    "w-4 h-4",
    "inline-block",
    "m-1.5px",
    "rounded-sm",
);

const BOX_ANIM: &str = build_classes!(
    "scale-100",
    "hover:scale-110",
    "transition-all",
    "duration-150",
    "hover:duration-75",
);

const NOTDAY_BASE: &str = build_classes!(
    "bg-opacity-70 hover:bg-opacity-90",
);

const NOTDAY_MAIN: &str = build_classes!(BOX_BASE, NOTDAY_BASE, BOX_ANIM, );

const BOX_ACTIVATED_NOTDAY: &str = build_classes!(NOTDAY_MAIN, "bg-blue-400");
const BOX_DEACTIVATED_NOTDAY: &str = build_classes!(NOTDAY_MAIN, "bg-slate-500");

const ISDAY_BASE: &str = build_classes!(
    "bg-opacity-80 hover:bg-opacity-100",
);
const ISDAY_MAIN: &str = build_classes!(BOX_BASE, ISDAY_BASE, BOX_ANIM);

const BOX_ACTIVATED_ISDAY: &str = build_classes!(ISDAY_MAIN, "bg-blue_400_saturated");
const BOX_DEACTIVATED_ISDAY: &str = build_classes!(ISDAY_MAIN, "bg-slate-400");

#[function_component(SectionDisplay)]
pub fn section_display(props: &SectionData) -> Html {
    let state = use_context::<UseReducerHandle<PageState>>().expect("no state ctx found");

    let day = if let &PageState::Main { day } = &*state { day } else {
        unimplemented!("Section display is (currently) unimplemented for the state {:#?}", state);
    };

    let (days, period) = match &props.guts {
        SectionDataGuts::Bad(s) => return html! {s},
        SectionDataGuts::Good {
            days,
            period
        } => (days, period),
    };

    let indicators = days
        .iter()
        .copied()
        .enumerate()
        .skip(1).take(5)
        .map(|(day_num, active)| {
            let state = state.clone();
            html! {
                <div
                    onclick={move |_| state.dispatch(StateUpdateAction::SetDayFilter(day_num))}
                    class={
                        if let Some(day) = day {
                            if day == day_num {
                                if active {BOX_ACTIVATED_ISDAY} else {BOX_DEACTIVATED_ISDAY}
                            } else {
                                if active {BOX_ACTIVATED_NOTDAY} else {BOX_DEACTIVATED_NOTDAY}
                            }
                        } else {
                            if active {BOX_ACTIVATED_NOTDAY} else {BOX_DEACTIVATED_NOTDAY}
                        }
                    }/>
            }
        }).collect::<Vec<Html>>();

    html! {
        <div class="h-full inline-flex items-center place-content-center flex-col px-1.5">
            <span class="font-title font-medium text-xl" >{"Per. "}{period}</span>
            <span class="pb-2">{if days != &[false; 7] {
                html! {<div>{indicators}</div>}
            } else {
                html! {}
            }}</span>
        </div>
    }
}

#[function_component(GradeIndicatorTooltip)]
pub fn grade_indicator_tooltip(props: &GradeIndicatorProps) -> Html {
    html! {
        <div class="text-sm absolute bottom-0 left-full bg-gray-500 px-2 w-max rounded-md inline-flex items-center justify-center h-6 target-div">
            {if props.enabled {"Affects GPA"} else {"Does not affect GPA"}}
        </div>
    }
}

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct GradeIndicatorProps {
    enabled: bool
}

const GRADE_INDICATOR_BASE: &str = build_classes!(
    "inline-flex w-6 h-6",
    "bg-gray-500",
    "rounded-md",
    "text-lg",
    "leading-none",
    "items-center",
    "justify-center",
    "mr-2",
    "text-gray-100",
    "trigger-div",
);

const GRADE_INDICATOR_ENABLED: &str = build_classes!(GRADE_INDICATOR_BASE);

const STRIKETHROUGH: &str = build_classes!(
    "relative before:absolute before:top-[calc(50%-1.5px)]",
    "before:w-full",
    "before:border-b-[3px] before:border-red-500 before:skew-y-[-30deg]"
);

const GRADE_INDICATOR_DISABLED: &str = build_classes!(GRADE_INDICATOR_BASE, "contrast-50", STRIKETHROUGH);

#[function_component(GradeIndicator)]
pub fn grade_indicator(props: &GradeIndicatorProps) -> Html {
    html! {
        <div class="relative p-0 text-xs">
            <div
                class={if props.enabled {
                    GRADE_INDICATOR_ENABLED
                } else {
                    GRADE_INDICATOR_DISABLED
                }}>
                {"G"}
            </div>
            <GradeIndicatorTooltip enabled={props.enabled} />
        </div>
    }
}


const CLASS_BASE: &str = build_classes!(
    "bg-opacity-0 bg-zinc-500 hover:bg-opacity-25",
    "[transition:background_200ms_ease-in-out_0s,height_300ms_ease-in-out_0s,transform_300ms_ease-in-out_0s]",
    "rounded-xl overflow-hidden",
);
const CLASS_SHOWING: &str = build_classes!(CLASS_BASE, "h-20 scale-y-100");
const CLASS_HIDDEN: &str = build_classes!(CLASS_BASE, "h-0 scale-y-0");


#[function_component(MainPageClass)]
pub fn main_page_class(props: &MainPageClassProps) -> Html {
    const MAIN_BODY: &str = build_classes!(
        "inline-flex flex-col relative",
        "ml-5",
        "before:h-[80%] before:top-[10%] before:absolute before:-left-4",
        "before:border-gray-500 before:border-[1px]",
    );

    let name = &props.entry.name;
    let is_ungraded = name.starts_with('~');
    let without_tilde = name.strip_prefix('~').unwrap_or(name);
    html! {
        <div class={if props.enabled {CLASS_SHOWING} else {CLASS_HIDDEN}}>
            <SectionDisplay guts={props.entry.section.guts.clone()} />
            <div class={MAIN_BODY}>
                <span class="flex flex-row text-2xl text-gray150 items-center">
                    <GradeIndicator enabled={!is_ungraded}/>
                    {without_tilde}
                </span>
                <span class="text-sm text-gray-400 rounded">{props.entry.id.0}</span>
            </div>
        </div>
    }
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
