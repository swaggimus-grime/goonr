use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use web_sys::wasm_bindgen::JsCast;
use yew::prelude::*;

mod components;

#[function_component(App)]
fn app() -> Html {
    let scene_path = use_state(|| "".to_string());
    let response = use_state(|| "".to_string());

    let on_submit = {
        let scene_path = scene_path.clone();
        let response = response.clone();

        Callback::from(move |_| {
            let path = scene_path.clone();
            let resp = response.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::json!({
                    "input_path": (*path).clone()
                });

                let result = reqwest::Client::new()
                    .post("http://localhost:3000/load_scene")
                    .json(&body)
                    .send()
                    .await;

                match result {
                    Ok(res) => {
                        let text = res.text().await.unwrap_or("Failed to read response".into());
                        resp.set(text);
                    }
                    Err(e) => resp.set(format!("Error: {e}")),
                }
            });
        })
    };

    html! {
        <div>
            <h1>{ "Load Scene" }</h1>
            <input
                type="text"
                placeholder="Enter path to .zip or directory"
                oninput={Callback::from(move |e: InputEvent| {
                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                    scene_path.set(input.value());
                })}
            />
            <button onclick={on_submit}>{ "Load" }</button>
            <p>{ (*response).clone() }</p>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}