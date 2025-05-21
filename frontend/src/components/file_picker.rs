use web_sys::{File, FileReader, HtmlInputElement};
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::prelude::Closure;
use yew::{function_component, html, use_node_ref, Callback, Html};

#[function_component(FilePicker)]
pub fn file_picker() -> Html {
    let file_input_ref = use_node_ref();
    let on_change = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_| {
            let input = file_input_ref.cast::<HtmlInputElement>().unwrap();
            if let Some(file) = input.files().and_then(|files| files.get(0)) {
                let file = File::from(file);
                let name = file.name();

                let reader = FileReader::new().unwrap();
                let onload = Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
                    log::info!("Loaded file: {}", name);
                    // In desktop-b, invoke tauri command
                    #[cfg(target_arch = "wasm32")]
                    gloo::dialogs::alert(&format!("Loaded file: {}", name));
                }) as Box<dyn FnMut(_)>);

                reader.read_as_array_buffer(&file).unwrap();
                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget(); // don't drop early
            }
        })
    };

    html! {
        <div>
            <input type="file" ref={file_input_ref} onchange={on_change} accept=".zip" />
        </div>
    }
}