use yew::{html, Html};
use yew_router::Routable;
use crate::components::viewer::Viewer;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/viewer/:name")]
    Viewer { name: String },
    #[at("/scenes")]
    Scenes,
}

pub(crate) fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { "Home" },
        Route::Viewer { name } => html! { <Viewer scene_name={name} /> },
        Route::Scenes => html! { "View scene metadata" },
    }
}