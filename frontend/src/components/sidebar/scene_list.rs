use gloo_console::info;
use stylist::yew::styled_component;
use web_sys::MouseEvent;
use yew::{html, Callback, Html, NodeRef, Properties, UseStateHandle};
use yew_router::prelude::use_navigator;
use web_cmn::scene::SceneResponse;
use crate::route::Route;

#[derive(Properties, PartialEq)]
pub struct SceneListProps {
    pub scenes: UseStateHandle<Vec<SceneResponse>>,
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
            for props.scenes.iter().map(|response| {
                let scene = response.clone();
                let navigator = navigator.clone();
                let on_click = Callback::from(move |_: MouseEvent| {
                    info!("Navigating to websplat with scene name: ", &scene.name);
                    navigator.push(&Route::Viewer { scene_name: scene.name.clone() });
                });

                let delete_scene = delete_scene.clone();
                let s = response.clone();
                let on_click_delete = Callback::from(move |e: MouseEvent| {
                    e.stop_propagation(); // Prevents navigation when clicking delete
                    delete_scene.emit(s.name.clone());
                });

                html! {
                    <div
                        class="mb-2 cursor-pointer rounded p-2 hover:bg-gray-200 transition flex items-center justify-between group"
                        onclick={on_click}
                    >
                        <span class="truncate">{ response.name.as_str() }</span>
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