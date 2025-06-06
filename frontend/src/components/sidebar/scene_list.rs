use gloo_console::info;
use stylist::yew::styled_component;
use uuid::Uuid;
use web_sys::MouseEvent;
use yew::{html, Callback, Html, NodeRef, Properties, UseStateHandle};
use yew_router::prelude::use_navigator;
use web_cmn::responses::scene::SceneMetadata;
use crate::route::Route;

#[derive(Properties, PartialEq)]
pub struct SceneListProps {
    pub scenes: UseStateHandle<Vec<SceneMetadata>>,
}

#[styled_component(SceneList)]
pub fn scene_list(props: &SceneListProps) -> Html {
    let navigator = use_navigator().unwrap();

    let delete_scene = {
        let scenes = props.scenes.clone();
        Callback::from(move |scene_name: String| {
            let new_list = scenes.iter().cloned().filter(|s| s.name != scene_name).collect();
            scenes.set(new_list);
        })
    };

    html! {
        {
            for props.scenes.iter().map(|metadata| {
                let scene = metadata.clone();
                let navigator = navigator.clone();
                let on_click = Callback::from(move |_: MouseEvent| {
                    info!("Navigating to viewer with scene name: {}", &scene.name);
                    navigator.push(&Route::Viewer { name: scene.name.clone() });
                });

                let delete_scene = delete_scene.clone();
                let s = metadata.clone();
                let on_click_delete = Callback::from(move |e: MouseEvent| {
                    e.stop_propagation(); // Prevents navigation when clicking delete
                    delete_scene.emit(s.name.clone());
                });

                html! {
                    <div
                        class="mb-2 cursor-pointer rounded p-2 hover:bg-gray-200 transition flex items-center justify-between group"
                        onclick={on_click}
                    >
                        <span class="truncate">{ metadata.name.as_str() }</span>
                        <button
                            class="text-red-500 opacity-0 group-hover:opacity-100 transition-opacity"
                            onclick={on_click_delete}
                            title="Delete Scene"
                        >
                            { "🗑️" }
                        </button>
                    </div>
                }
            })
        }
    }
}