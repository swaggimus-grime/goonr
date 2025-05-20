use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use web_sys::wasm_bindgen::JsCast;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let canvas_ref = use_node_ref();

    use_effect_with(canvas_ref.clone(), move |canvas_ref| {
        if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
            let gl: WebGl2RenderingContext = canvas
                .get_context("webgl2")
                .unwrap()
                .unwrap()
                .dyn_into()
                .unwrap();

            // TODO: Setup shaders, buffers, draw loop
            gl.clear_color(0.1, 0.1, 0.1, 1.0);
            gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        }
        || ()
    });

    html! {
        <div>
            <h1>{ "Gaussian Splatting Viewer" }</h1>
            <canvas ref={canvas_ref} width="800" height="600" style="border: 1px solid black;" />
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}