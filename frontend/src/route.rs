use yew::{html, Html};
use yew_router::Routable;
use crate::components::viewer::Viewer;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/websplat/:scene_name")]
    Viewer { scene_name: String },
    #[at("/scenes")]
    Scenes,
}

pub(crate) fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { "Home" },
        Route::Viewer { scene_name } => html! { <Viewer scene_name={scene_name} /> },
        Route::Scenes => html! { "View scene-source metadata" },
    }
}