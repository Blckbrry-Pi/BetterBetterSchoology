use bbs_shared::{data::{ClassEntry, SectionData, SectionDataGuts, OptMutComponent, Keyed, Assignment}, PageState, StateUpdateAction, ClassID, MaterialID};
use web_sys::MouseEvent;
use yew::{function_component, Properties, html, Html, use_context, UseReducerHandle, Callback};
use web_sys::{window, console};

use crate::build_classes;

#[derive(Debug, Properties, PartialEq)]
pub struct MaterialTypeProps {
    material_type: String,
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

#[function_component(MaterialTypeDisplay)]
pub fn material_type_display(props: &MaterialTypeProps) -> Html {
    html! {
        <div
            class="h-full inline-flex items-center justify-center flex-col px-5 w-36"
            onclick={|event: MouseEvent| {
                event.prevent_default();
                event.stop_propagation();
            }}>
            {"this is an "}{props.material_type.clone()}
        </div>
    }
}


#[derive(Debug, Properties, PartialEq)]
pub struct ClassPageMaterialProps {
    pub assignment_data: Assignment,
    pub into_material_callback: Callback<MaterialID>,  
}


const CLASS_BASE: &str = build_classes!(
    "bg-opacity-0 bg-zinc-500 hover:bg-opacity-25",
    "[transition:background_200ms_ease-in-out_0s,height_300ms_ease-in-out_0s,transform_300ms_ease-in-out_0s]",
    "rounded-xl overflow-hidden",
    "h-20 scale-y-100",
);


#[function_component(ClassPageMaterial)]
pub fn class_page_material(props: &ClassPageMaterialProps) -> Html {
    const MAIN_BODY: &str = build_classes!(
        "inline-flex flex-col relative",
        "ml-5",
        "before:h-[80%] before:top-[10%] before:absolute before:-left-4",
        "before:border-gray-500 before:border-[1px]",
    );


    let title = &props.assignment_data.title;
    let id = MaterialID(u64::from_str_radix("0", 10).unwrap());
    let body = &props.assignment_data.body;
    let kind = &props.assignment_data.kind;
    let callback = props.into_material_callback.clone();

    html! {
        <div
            class={CLASS_BASE}
            onclick={move |_| callback.clone().emit(id)}>
            <MaterialTypeDisplay material_type={kind.clone()} />
            <div class={MAIN_BODY}>
                <span class="flex flex-row text-2xl text-gray150 items-center">
                    {title}
                </span>
                <span class="text-sm text-gray-400 rounded">{body}</span>
            </div>
        </div>
    }
}

#[derive(Debug, Properties, PartialEq)]
pub struct ClassPageProps {
    pub materials: Keyed<OptMutComponent<Vec<Assignment>>>,
}


#[function_component(ClassPage)]
pub fn class_page(props: &ClassPageProps) -> Html {
    let state = use_context::<UseReducerHandle<PageState>>().expect("no state ctx found");

    let materials_ref = props.materials.borrow();
    
    let material_html = match materials_ref.as_ref() {
        Some(a) => html! {
            {
                if a.len() == 0 {
                    html! {
                        <div class="text-center text-gray-500">
                            {"No materials found"}
                        </div>
                    }
                } else {
                    a
                    .iter()
                    .map(|entry| {
                        let state = state.clone();
                        console::error_1(&format!("{:#?}", a).into());
                        html! {
                            <ClassPageMaterial
                                assignment_data={entry.clone()}
                                key={entry.id.0}
                                into_material_callback={Callback::from(move |id| state.clone().dispatch(StateUpdateAction::ToClassItem(id)))}/>
                        }
                    })
                    .collect::<Html>()
                }
            }
        },
        None => html! {
            <h1 class="ml-7">{"Loading..."}</h1>
        },
    };

    html! {
        <div>
            {material_html}
        </div>
    }
}




#[derive(Debug, Properties, PartialEq)]
pub struct ClassPageOverlayProps {
    pub loading: bool,
    pub error: Option<()>,
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


#[function_component(ClassPageOverlay)]
pub fn class_page_overlay(props: &ClassPageOverlayProps) -> Html {
    let message = match props.error {
        Some(_) => html! {
            <>
                <h1>{"Error lol"}</h1>
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
                        html! {<button onclick={move |_| callback.emit(())} class="mt-3 px-3 py-2 bg-violet-400 rounded-md text-black">{"<-- Return to main page"}</button>}
                    } else {
                        html! {}
                    }
                }
            </div>
        </div>
    }
}
