use gloo_console::log;
use gloo_net::http::Request;
use serde::Deserialize;
use stylist::yew::styled_component;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, File, HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub on_scene_uploaded: Callback<SceneMetadata>,
    pub scenes: Vec<SceneMetadata>,
    pub on_select_scene: Callback<String>,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct SceneMetadata {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[styled_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    let file_input_ref = use_node_ref();

    let on_upload_click = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_file_change = {
        let on_scene_uploaded = props.on_scene_uploaded.clone();
        Callback::from(move |event: Event| {
            let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let file_clone = file.clone();
                    let on_scene_uploaded = on_scene_uploaded.clone();

                    spawn_local(async move {
                        let form_data = web_sys::FormData::new().unwrap();
                        form_data.append_with_blob("scene_zip", &file_clone).unwrap();

                        match Request::post("/api/upload_scene")
                            .body(form_data)
                            .expect("Failed to build request")
                            .send()
                            .await
                        {
                            Ok(response) if response.ok() => {
                                let metadata: SceneMetadata = response
                                    .json()
                                    .await
                                    .expect("Failed to parse JSON");
                                on_scene_uploaded.emit(metadata);
                            }
                            Ok(response) => {
                                log!("Upload failed. Status:", response.status());
                            }
                            Err(err) => {
                                log!("Upload error:", err.to_string());
                            }
                        }
                    });
                }
            }
        })
    };

    let on_scene_select = {
        let on_select_scene = props.on_select_scene.clone();
        Callback::from(move |e: Event| {
            let select = e.target().unwrap().dyn_into::<HtmlSelectElement>().unwrap();
            on_select_scene.emit(select.value());
        })
    };

    html! {
        <div class="space-y-6 font-frutiger">
            <h2 class="text-2xl font-bold text-aeroBlue drop-shadow-glass">{ "Scene Manager" }</h2>
    
            <div class="space-y-2">
                <label class="block text-sm text-gray-700 dark:text-gray-300">{ "Upload a new scene" }</label>
                <button
                    onclick={on_upload_click}
                    class="w-full bg-aeroPurple hover:bg-aeroPink text-white font-medium py-2 px-4 rounded-xl shadow-glass transition-all"
                >
                    { "Upload" }
                </button>
                <input
                    ref={file_input_ref}
                    type="file"
                    style="display: none;"
                    onchange={on_file_change}
                />
            </div>
    
            <div class="space-y-2">
                <label class="block text-sm text-gray-700 dark:text-gray-300">{ "Select existing scene" }</label>
                <select
                    onchange={on_scene_select}
                    class="w-full bg-white/10 hover:bg-white/20 backdrop-blur-xs text-gray-900 dark:text-white p-2 rounded-xl shadow-glass transition-all"
                >
                    <option value="" disabled=true selected=true>{ "Choose a scene" }</option>
                    { for props.scenes.iter().map(|scene| html! {
                        <option value={scene.id.clone()}>{ &scene.name }</option>
                    }) }
                </select>
            </div>
        </div>
    }
}
