use sidebar::yew::item::MenuItem;
use sidebar::yew::submenu::{Submenu, SubmenuProps};
use stylist::yew::styled_component;
use yew::{html, Html, Properties, UseStateHandle};

#[derive(Properties, PartialEq)]
pub struct ScenesSubmenuProps {
    pub selected: UseStateHandle<String>
}

#[styled_component(ScenesSubmenu)]
pub fn scenes_submenu(props: &ScenesSubmenuProps) -> Html {
    html! {
        <Submenu title="Scenes" icon_html={html! {<span>{ "🏞️" }</span>}}>
            <MenuItem
                label="Upload"
                href=""
                icon_html={html! {<span>{ "➕" }</span>}}
                selected={props.selected.clone()}
            />
        </Submenu>
    }
}