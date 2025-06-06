use stylist::yew::styled_component;
use yew::{html, Html};

#[styled_component(ScenesPage)]
pub fn scene_mgr() -> Html {
    html! { 
        <h1>{ "Scenes" }</h1> 
    }
}