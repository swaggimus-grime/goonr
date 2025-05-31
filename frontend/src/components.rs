mod sidebar;
mod viewer_canvas;

use stylist::yew::styled_component;
use yew::{html, use_state, Callback, Html};
use web_cmn::responses::scene::SceneMetadata;
use crate::components::sidebar::{Sidebar};
use crate::components::viewer_canvas::ViewerCanvas;

#[styled_component(App)]
pub fn app() -> Html {
    let scenes = use_state(|| vec![]);
    let selected_scene_id = use_state(|| None::<String>);

    let on_select_scene = {
        let selected_scene_id = selected_scene_id.clone();
        Callback::from(move |id: String| {
            selected_scene_id.set(Some(id));
        })
    };

    let on_scene_uploaded = {
        let scenes = scenes.clone();
        Callback::from(move |metadata: SceneMetadata| {
            let mut new_scenes = (*scenes).clone();
            new_scenes.push(metadata);
            scenes.set(new_scenes);
        })
    };

    html! {
        <div class="min-h-screen bg-gradient-to-br from-[#c0f0ff] via-[#a0e0ff] to-[#c8ffe0] font-frutiger text-gray-900 dark:text-white p-8">
            <div class="max-w-6xl mx-auto space-y-6">
                <h1 class="text-5xl font-bold text-aeroPurple drop-shadow-glass text-center text-gray-900">
                    {"Goonr Viewer"}
                </h1>

                <div class="bg-aeroGlass backdrop-blur-xs rounded-xl shadow-glass p-6 border border-white/20 text-gray-900 dark:text-white">
                    <Sidebar
                        on_scene_uploaded={on_scene_uploaded}
                        scenes={(*scenes).clone()}
                        on_select_scene={on_select_scene}
                    />
                    
                    <div class="mt-6">
                        {
                            if let Some(scene_id) = (*selected_scene_id).clone() {
                                html! { <ViewerCanvas scene_id={scene_id} /> }
                            } else {
                                html! {
                                    <div class="flex items-center justify-center h-64 text-lg italic text-gray-200">
                                        {"Select a scene to begin viewing."}
                                    </div>
                                }
                            }
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}