use yew::prelude::*;

mod components;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div class="flex h-screen w-screen bg-gray-950 text-white font-sans">
            <components::Sidebar />
            <components::ViewerCanvas />
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}