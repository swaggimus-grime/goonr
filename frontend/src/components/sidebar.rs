mod scene_list;
mod scene_upload_btn;

use std::path::PathBuf;
use gloo_console::info;
use stylist::yew::styled_component;
use wasm_bindgen::JsCast;
use web_sys::{File, HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_router::prelude::Link;
use web_cmn::scene::SceneResponse;
use crate::components::sidebar::scene_list::{SceneList};
use crate::components::sidebar::scene_upload_btn::{SceneUploadBtn};
use crate::route::Route;

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub on_upload_click: Callback<MouseEvent>,
    pub scenes: UseStateHandle<Vec<SceneResponse>>,
}

#[styled_component(MainSidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    let collapsed = use_state(|| true); // Start collapsed like YouTube

    let toggle = {
        let collapsed = collapsed.clone();
        Callback::from(move |_| collapsed.set(!*collapsed))
    };
    
    html! {
        <aside class={classes!(
            "relative",
            "z-20",
            "h-screen",
            "bg-green",
            "border-r",
            "shadow",
            "flex",
            "flex-col",
            "transition-all",
            "duration-300",
            "ease-in-out",
            "overflow-hidden",
            if *collapsed { "w-16" } else { "w-60" }
        )}>
            // Top hamburger
            <div class="p-4">
                <button onclick={toggle} class="text-xl hover:bg-gray-200 p-2 rounded transition">
                    { "≡" }
                </button>
            </div>

            <nav class="flex-1 px-2 space-y-2">
                if *collapsed {
                    <>
                        { nav_icon(Route::Home, "🏠") }
                        { nav_icon(Route::Scenes, "🔥") }
                    </>
                } else {
                    <>
                        { nav_item(Route::Home, "🏠", "Home") }
                        { nav_item(Route::Scenes, "📚", "Scenes") }
                        <hr class="my-2 border-gray-300" />
                        <div class="mt-4 px-2 overflow-auto max-h-64">
                            <SceneUploadBtn
                                scenes = {props.scenes.clone()}
                                on_click = {props.on_upload_click.clone()}
                            />
                        </div>
                        <hr class="my-2 border-gray-300" />
                        <div class="mt-4 px-2 overflow-auto max-h-64">
                            <SceneList
                                scenes = {props.scenes.clone()}
                            />
                        </div>
                    </>
                }
            </nav>
        </aside>
    }
}

// Minimal icon-only nav link (for collapsed sidebar)
fn nav_icon(route: Route, icon: &str) -> Html {
    html! {
        <Link<Route> to={route}>
            <div class="flex justify-center p-2 rounded hover:bg-gray-100 transition cursor-pointer">
                <span class="text-xl">{ icon }</span>
            </div>
        </Link<Route>>
    }
}

// Full nav item (for expanded sidebar)
fn nav_item(route: Route, icon: &str, label: &str) -> Html {
    html! {
        <Link<Route> to={route}>
            <div class="flex items-center space-x-4 p-2 rounded hover:bg-gray-100 cursor-pointer transition">
                <span class="text-xl">{ icon }</span>
                <span class="text-sm">{ label }</span>
            </div>
        </Link<Route>>
    }
}
