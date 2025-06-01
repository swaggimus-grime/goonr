use stylist::yew::styled_component;
use yew::{html, Html};

#[styled_component(ScenesPage)]
pub fn scenes_page() -> Html {
    html! { 
        <h1>{ "Scenes" }</h1> 
    }
}