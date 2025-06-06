use std::ops::Deref;
use std::path::PathBuf;
use gloo_console::log;
use gloo_net::Error;
use gloo_net::http::{Request, Response};
use stylist::yew::styled_component;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{Event, File, HtmlInputElement, MouseEvent};
use yew::{html, Callback, Html, NodeRef, Properties, UseStateHandle};
use yew::platform::spawn_local;
use web_cmn::responses::scene::SceneMetadata;

#[derive(Properties, PartialEq)]
pub struct SceneUploadProps {
    pub file_input_ref: NodeRef,
    pub scenes: UseStateHandle<Vec<SceneMetadata>>,
}

#[styled_component(SceneUpload)]
pub fn scene_upload(props: &SceneUploadProps) -> Html {
    let on_upload_click = {
        let file_input_ref = props.file_input_ref.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click(); // opens file browser
            }
        })
    };

    let on_file_change = {
        let mut scenes = props.scenes.clone();
        Callback::from(move |event: Event| {
            let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    if file.name().ends_with(".zip") {
                        send_upload_request(file, scenes.clone());
                    }
                }
            }
        })
    };
    
    html! {
         <>
             <input
                 type="file"
                 accept=".zip"
                 ref={props.file_input_ref.clone()}
                 onchange={on_file_change}
                 class="hidden"
                 style="display: none;"
             />
             <div
                 class="mb-2 cursor-pointer rounded p-2 bg-blue-100 hover:bg-blue-200 text-center font-semibold transition"
                 onclick={on_upload_click}
             >
                 { "+ Upload Scene" }
             </div>
         </>
     }
}

fn send_upload_request(file: File, mut scenes: UseStateHandle<Vec<SceneMetadata>>) {
    spawn_local(async move {
        let form_data = web_sys::FormData::new().unwrap();
        form_data.append_with_blob("scene_zip", file.as_ref()).unwrap();

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
                scenes.set({
                    let mut new_scenes = scenes.deref().clone();
                    new_scenes.push(metadata);
                    new_scenes
                });
            },
            Ok(response) => {
                log!("Upload failed. Status:", response.status());
            },
            Err(err) => {
                log!("Upload error:", err.to_string());
            }
        }
    });
}