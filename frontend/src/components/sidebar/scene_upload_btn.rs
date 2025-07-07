use stylist::yew::styled_component;
use web_sys::MouseEvent;
use yew::{html, use_state, Callback, Html, Properties, UseStateHandle};
use yew_router::hooks::use_navigator;
use web_cmn::scene::{SceneResponse};
use crate::components::forms::scene_upload::SceneUploadModal;

#[derive(Properties, PartialEq)]
pub struct SceneUploadBtnProps {
    pub scenes: UseStateHandle<Vec<SceneResponse>>,
    pub on_click: Callback<MouseEvent>,
}

#[styled_component(SceneUploadBtn)]
pub fn scene_upload_btn(props: &SceneUploadBtnProps) -> Html {
    let on_scene_uploaded = {
        let on_click = props.on_click.clone();
        Callback::from(move |name: String| {
            // Call backend or store
            gloo::console::log!(format!("Uploading scene: {}", name));
        })
    };

    html! {
        <>
            <button onclick={props.on_click.clone()}>
                {"Upload"}
            </button>
        </>
    }
}
