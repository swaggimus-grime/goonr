use gloo::utils::format::JsValueSerdeExt;
use stylist::yew::styled_component;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{js_sys, HtmlInputElement};
use yew::prelude::*;
use web_cmn::scene::SceneResponse;

#[derive(Properties, PartialEq)]
pub struct SceneUploadFormProps {
    pub on_close: Callback<()>,
    pub on_scene_uploaded: Callback<SceneResponse>,
}

#[styled_component(SceneUploadModal)]
pub fn upload_modal(props: &SceneUploadFormProps) -> Html {
    let file_input_ref = use_node_ref();
    let name = use_state(|| "".to_string());
    let upload_mode = use_state(|| "folder".to_string()); // "folder" or "zip"

    let on_name_change = {
        let name = name.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            name.set(input.value());
        })
    };

    let on_mode_change = {
        let upload_mode = upload_mode.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            upload_mode.set(input.value());
        })
    };

    let on_submit = {
        let name = name.clone();
        let file_input_ref = file_input_ref.clone();
        let on_scene_uploaded = props.on_scene_uploaded.clone();
        let on_close = props.on_close.clone();

        use_callback(
            (name, file_input_ref, on_scene_uploaded, on_close),
            move |_, (name, file_input_ref, on_scene_uploaded, on_close)| {
                let name = (*name).clone();

                let Some(input) = file_input_ref.cast::<HtmlInputElement>() else { return };
                let Some(files) = input.files() else { return };

                let mut form = web_sys::FormData::new().unwrap();
                form.append_with_str("name", &name).unwrap();

                for i in 0..files.length() {
                    if let Some(file) = files.item(i) {
                        let path = get_webkit_relative_path(&file);
                        let filename = if path.is_empty() { file.name() } else { path };
                        form.append_with_blob_and_filename("file", &file, &filename).unwrap();
                    }
                }

                let mut request = web_sys::RequestInit::new();
                request.set_method("POST");
                request.set_body(&form);

                let window = web_sys::window().unwrap();
                let request = web_sys::Request::new_with_str_and_init("/api/upload_scene", &request).unwrap();

                let on_scene_uploaded = on_scene_uploaded.clone();
                let on_close = on_close.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let fetch_promise = window.fetch_with_request(&request);
                    let resp_result = wasm_bindgen_futures::JsFuture::from(fetch_promise).await;

                    match resp_result {
                        Ok(resp_value) => {
                            let resp: web_sys::Response = resp_value.dyn_into().unwrap();
                            if resp.ok() {
                                let json_promise = resp.json().unwrap();
                                let json_value = wasm_bindgen_futures::JsFuture::from(json_promise).await;

                                match json_value {
                                    Ok(js_value) => {
                                        let scene = js_value.into_serde::<SceneResponse>();
                                        match scene {
                                            Ok(scene) => on_scene_uploaded.emit(scene),
                                            Err(e) => gloo_console::error!(format!("JSON parsing failed: {:?}", e))
                                        }
                                    }
                                    Err(e) => {
                                        gloo_console::error!(format!("JSON parsing failed: {:?}", e));
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            gloo_console::error!(format!("Upload failed: {:?}", err));
                        }
                    }

                    on_close.emit(());
                });
            },
        )
    };

    html! {
        <div class="bg-white rounded-2xl shadow-xl p-6 w-full max-w-md">
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-glass p-6 w-full max-w-md">
                <h2 class="text-2xl font-bold mb-4 text-center">{"Upload Scene"}</h2>
                <input
                    type="text"
                    placeholder="Scene name"
                    value={(*name).clone()}
                    oninput={on_name_change}
                    class="w-full p-2 mb-4 border border-gray-300 rounded text-gray-900 bg-white"
                />

                <div class="mb-4">
                    <label class="mr-4">
                        <input
                            type="radio"
                            name="upload_mode"
                            value="folder"
                            checked={*upload_mode == "folder"}
                            onchange={on_mode_change.clone()}
                        />
                        {" Folder"}
                    </label>
                    <label>
                        <input
                            type="radio"
                            name="upload_mode"
                            value="zip"
                            checked={*upload_mode == "zip"}
                            onchange={on_mode_change}
                        />
                        {" .zip File"}
                    </label>
                </div>

                <input
                    type="file"
                    ref={file_input_ref.clone()}
                    multiple={*upload_mode == "folder"}
                    // Only add `webkitdirectory` if mode is folder
                    webkitdirectory={*upload_mode == "folder"}
                    accept={if *upload_mode == "zip" { ".zip" } else { "" }}
                    class="w-full mb-4"
                />

                <div class="flex justify-end gap-2">
                    <button onclick={on_submit} class="px-4 py-2 bg-aeroPurple text-white rounded hover:bg-opacity-90 transition">
                        {"Upload"}
                    </button>
                    <button onclick={props.on_close.reform(|_| ())} class="px-4 py-2 bg-gray-300 text-gray-900 rounded hover:bg-gray-400 transition">
                        {"Cancel"}
                    </button>
                </div>
            </div>
        </div>
    }
}

fn get_webkit_relative_path(file: &web_sys::File) -> String {
    js_sys::Reflect::get(file.as_ref(), &JsValue::from_str("webkitRelativePath"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| file.name())
}
