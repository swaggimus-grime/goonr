use yew::prelude::*;

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    html! {
        <div>
            <h2 class="text-xl font-bold mb-4">{ "Settings" }</h2>
            <div class="space-y-2">
                <button class="w-full bg-blue-600 hover:bg-blue-700 text-white py-2 px-4 rounded">{ "Load Model" }</button>
                <button class="w-full bg-green-600 hover:bg-green-700 text-white py-2 px-4 rounded">{ "Reset View" }</button>
            </div>
        </div>
    }
} 
