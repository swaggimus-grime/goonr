use gloo_console::log;
use gloo_net::http::Request;
use serde::Deserialize;
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

#[function_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    let file_input_ref = use_node_ref();

    let on_upload_click = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click(); // Trigger hidden file input
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
        <div class="w-64 bg-gray-900 p-4 border-r border-gray-800">
            <h1 class="text-xl font-bold mb-6">{"Goonr Viewer"}</h1>

            <div class="space-y-4">
                <div>
                    <label class="block text-sm mb-1 text-gray-400">{"Load Scene"}</label>
                    <button
                        onclick={on_upload_click}
                        class="w-full bg-blue-600 hover:bg-blue-700 text-white py-2 px-3 rounded"
                    >
                        {"Upload"}
                    </button>
                    <input
                        ref={file_input_ref}
                        type="file"
                        style="display: none;" // 👈 Completely hide file input
                        onchange={on_file_change}
                    />
                </div>

                <div>
                    <label class="block text-sm mb-1 text-gray-400">{"Select Scene"}</label>
                    <select onchange={on_scene_select} class="w-full bg-gray-800 text-white p-2 rounded">
                        <option value="" disabled=true selected=true>{"Choose a scene"}</option>
                        { for props.scenes.iter().map(|scene| html! {
                            <option value={scene.id.clone()}>{ &scene.name }</option>
                        })}
                    </select>
                </div>
            </div>
        </div>
    }
}
