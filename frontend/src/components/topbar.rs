use yew::prelude::*;

#[function_component(Topbar)]
pub fn topbar() -> Html {
    html! {
        <div class="w-full bg-gray-700 p-2 text-center text-sm text-gray-300">
            { "Gaussian Splatting Viewer - Web Edition" }
        </div>
    }
}
