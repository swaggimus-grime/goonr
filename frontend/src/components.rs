mod sidebar;
pub(crate) mod viewer_canvas;
mod scene_manager;
mod forms;

use gloo_console::log;
use stylist::yew::styled_component;
use wasm_bindgen_futures::spawn_local;
use yew::{html, use_effect_with, use_state, Callback, Html};
use yew_router::{BrowserRouter, Switch};
use web_cmn::scene::{SceneResponse};
use crate::components::forms::scene_upload::SceneUploadModal;
use crate::components::sidebar::MainSidebar;
use crate::route;
use crate::route::Route;
use crate::services::scene::fetch_scenes;

#[styled_component(App)]
pub fn app() -> Html {
    let scenes = use_state(|| {
        vec![]
    });
    {
        let scenes = scenes.clone();
        use_effect_with((),
            move |_| {
                spawn_local(async move {
                    match fetch_scenes().await {
                        Ok(fetched) => scenes.set(fetched),
                        Err(err) => log!(format!("Failed to fetch scenes: {err:?}")),
                    }
                });
                || ()
            }
        );
    }
    let selected_scene_id = use_state(|| None::<String>);
    let show_upload_modal = use_state(|| false);

    let on_scene_uploaded = {
        let scenes = scenes.clone();
        Callback::from(move |response: SceneResponse| {
            scenes.set({
                let mut new_scenes = (*scenes).clone();
                new_scenes.push(response);
                new_scenes
            });
        })
    };

    html! {
        <>
            <div class="min-h-screen bg-gradient-to-br from-[#c0f0ff] via-[#a0e0ff] to-[#c8ffe0] font-frutiger text-gray-900 dark:text-white p-8">
                <BrowserRouter>
                    <div class="flex">
                        <MainSidebar
                            scenes = {scenes.clone()}
                            on_upload_click={Callback::from({
                                let show_upload_modal = show_upload_modal.clone();
                                move |_| show_upload_modal.set(true)
                            })}
                        />
                        <main class="flex-1 p-4 bg-gray-50">
                            <Switch<Route> render={route::switch} />
                        </main>
                    </div>
                </BrowserRouter>
            </div>
            {
                if *show_upload_modal {
                    html! {
                        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
                            <SceneUploadModal 
                                on_close = {
                                    Callback::from({
                                        let show_upload_modal = show_upload_modal.clone();
                                        move |_| show_upload_modal.set(false)
                                    })
                                }
                                on_scene_uploaded = {on_scene_uploaded.clone()}
                            />
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </>
    }
}