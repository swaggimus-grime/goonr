use gloo::utils::document;
use log::info;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{window, FormData, HtmlInputElement, Request, RequestInit, RequestMode, Response};
use yew::prelude::*;

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    // States for feedback
    let loading = use_state(|| false);
    let success = use_state(|| None::<String>);
    let error = use_state(|| None::<String>);

    // open_file_browser callback triggers click on hidden file input
    let open_file_browser = {
        Callback::from(|_| {
            if let Some(input_element) = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("scene-upload")
            {
                let input: HtmlInputElement = input_element.dyn_into().unwrap();
                input.click();
            }
        })
    };

    let on_file_selected = {
        info!("Entering on_file_selected callback");
        let loading = loading.clone();
        let success = success.clone();
        let error = error.clone();

        Callback::from(move |event: Event| {
            let input: HtmlInputElement = event.target_unchecked_into();

            if let Some(file) = input.files().and_then(|files| files.get(0)) {
                // Clone inside the async block to avoid making outer closure FnOnce
                let loading = loading.clone();
                let success = success.clone();
                let error = error.clone();
                let file = file.clone();

                spawn_local(async move {
                    loading.set(true);
                    success.set(None);
                    error.set(None);

                    // Do NOT set any headers manually here
                    let window = window().unwrap();

                    let form_data = FormData::new().expect("Failed to create FormData");
                    form_data.append_with_blob("scene_zip", &file).expect("Failed to append file");

                    let mut opts = RequestInit::new();
                    opts.method("POST");
                    opts.body(Some(&form_data));
                    opts.mode(RequestMode::Cors); // optional: depending on your backend CORS setup
                    
                    // Create request
                    let request = Request::new_with_str_and_init("/api/upload_scene", &opts)
                        .expect("Failed to create request");
                    request
                        .headers()
                        .set("Accept", "application/json")
                        .expect("Failed to set headers");

                    info!("Uploading file: {:?}", file);
                    match JsFuture::from(window.fetch_with_request(&request)).await {
                        Ok(resp_value) => {
                            let resp: Response = resp_value.dyn_into().unwrap();
                            if resp.ok() {
                                success.set(Some("Upload successful!".into()));
                            } else {
                                error.set(Some(format!("Upload failed: {}", resp.status_text())));
                            }
                        }
                        Err(err) => {
                            error.set(Some(format!("Fetch error: {:?}", err)));
                        }
                    }
                    loading.set(false);
                });
            }
        })
    };

    html! {
        <div class="w-64 bg-gray-900 p-4 border-r border-gray-800">
            <h1 class="text-xl font-bold mb-6">{"Goonr Splat Viewer"}</h1>

            <div class="space-y-6">
                <div>
                    <label class="block text-sm mb-1 text-gray-400">{"COLMAP Scene (.zip)"}</label>
                    <button
                        class="w-full bg-blue-600 hover:bg-blue-700 text-white py-2 px-3 rounded"
                        onclick={open_file_browser}
                        disabled={*loading}
                    >
                        { if *loading { "Uploading..." } else { "Upload COLMAP Zip" } }
                    </button>
                    <input
                        type="file"
                        accept=".zip"
                        id="scene-upload"
                        style="display: none;"
                        onchange={on_file_selected}
                        disabled={*loading}
                    />
                    {
                        if let Some(msg) = &*success {
                            html!{ <p class="text-green-400 mt-2">{ msg }</p> }
                        } else if let Some(err) = &*error {
                            html!{ <p class="text-red-500 mt-2">{ err }</p> }
                        } else {
                            html!{}
                        }
                    }
                </div>

                // ... your other UI elements ...
            </div>
        </div>
    }
}