use yew::prelude::*;

mod components;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class="flex h-screen w-screen bg-gray-900 text-white">
            <div class="w-64 bg-gray-800 p-4 overflow-y-auto">
                <components::Sidebar />
            </div>
            <div class="flex-1 relative">
                <components::Topbar />
                <components::ViewerCanvas />
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}