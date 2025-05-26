use uuid::Uuid;
use yew::prelude::*;

mod components;
mod route;

use components::*;


#[function_component(App)]
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
        <div class="flex h-screen w-screen bg-gray-950 text-white font-sans">
            <Sidebar
                on_scene_uploaded={on_scene_uploaded}
                scenes={(*scenes).clone()}
                on_select_scene={on_select_scene}
            />
            {
                if let Some(scene_id) = (*selected_scene_id).clone() {
                    html! { <ViewerCanvas scene_id={scene_id} /> }
                } else {
                    html! {
                        <div class="flex-1 flex items-center justify-center text-gray-500">
                            {"Select a scene"}
                        </div>
                    }
                }
            }
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}