mod sidebar;
pub(crate) mod viewer;
mod scene_manager;

use stylist::yew::styled_component;
use yew::{html, use_state, Callback, Html};
use yew_router::{BrowserRouter, Switch};
use web_cmn::responses::scene::SceneMetadata;
use crate::components::sidebar::MainSidebar;
use crate::components::viewer::Viewer;
use crate::route;
use crate::route::Route;

#[styled_component(App)]
pub fn app() -> Html {
    let scenes = use_state(|| vec![]);
    let selected_scene_name = use_state(|| None::<String>);

    let on_select_scene = {
        let selected_scene_name = selected_scene_name.clone();
        Callback::from(move |name: String| {
            selected_scene_name.set(Some(name));
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
            <BrowserRouter>
                <div class="flex">
                    <MainSidebar />
                    <main class="flex-1 p-4 bg-gray-50">
                        <Switch<Route> render={route::switch} />
                    </main>
                </div>
            </BrowserRouter>
            <div class="max-w-6xl mx-auto space-y-6">
                <h1 class="text-5xl font-bold text-aeroPurple drop-shadow-glass text-center text-gray-900">
                    {"Goonr"}
                </h1>

                <div class="bg-aeroGlass backdrop-blur-xs rounded-xl shadow-glass p-6 border border-white/20 text-gray-900 dark:text-white">
                    <div class="mt-6">
                        {
                            if let Some(scene_name) = (*selected_scene_name).clone() {
                                html! { <Viewer scene_name={scene_name} /> }
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