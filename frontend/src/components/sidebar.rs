mod scenes;

use gloo_console::info;
use sidebar::yew::sidebar::Sidebar;
use sidebar::yew::item::MenuItem;
use sidebar::yew::menu::Menu;
use sidebar::yew::submenu::Submenu;
use stylist::yew::styled_component;
use web_sys::{File, HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use web_cmn::responses::scene::SceneMetadata;
use crate::components::sidebar::scenes::ScenesSubmenu;

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub on_scene_uploaded: Callback<SceneMetadata>,
    pub scenes: Vec<SceneMetadata>,
    pub on_select_scene: Callback<String>,
}

#[styled_component(MainSidebar)]
pub fn sidebar() -> Html {
    let selected = use_state(|| String::from("Scenes"));

    html! {
        <Sidebar
            logo_img_url="static/logo.svg"
            logo_href="/"
        >
            <Menu sub_heading="Main">
                <ScenesSubmenu
                    selected={selected.clone()}
                />
                <MenuItem
                    label="Settings"
                    href="/settings"
                    icon_html={html! {<span>{ "⚙️" }</span>}}
                    selected={selected.clone()}
                />
            </Menu>
        </Sidebar>
    }
    /*
    let file_input_ref = use_node_ref();

    let is_open = use_state(|| true);

    let toggle = {
        let is_open = is_open.clone();
        Callback::from(move |_: MouseEvent| is_open.set(!*is_open))
    };

    let on_upload_click = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_file_change = {
        let on_scene_uploaded = props.on_scene_uploaded.clone();
        Callback::from(move |event: Event| {
            let input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let file_clone = file.clone();
                    let on_scene_uploaded = on_scene_uploaded.clone();

                    spawn_local(async move {
                        let form_data = web_sys::FormData::new().unwrap();
                        form_data.append_with_blob("scene_zip", &file_clone).unwrap();

                        match Request::post("/api/upload_scene")
                            .body(form_data)
                            .expect("Failed to build request")
                            .send()
                            .await
                        {
                            Ok(response) if response.ok() => {
                                let metadata: SceneMetadata = response
                                    .json()
                                    .await
                                    .expect("Failed to parse JSON");
                                on_scene_uploaded.emit(metadata);
                            }
                            Ok(response) => {
                                log!("Upload failed. Status:", response.status());
                            }
                            Err(err) => {
                                log!("Upload error:", err.to_string());
                            }
                        }
                    });
                }
            }
        })
    };

    let on_scene_select = {
        let on_select_scene = props.on_select_scene.clone();
        Callback::from(move |e: Event| {
            let select = e.target().unwrap().dyn_into::<HtmlSelectElement>().unwrap();
            on_select_scene.emit(select.value());
        })
    };

    html! {
        <div class="flex">
            <div class={classes!(
                "transition-all",
                "duration-300",
                "bg-gray-800",
                "text-white",
                "h-screen",
                "p-4",
                if *is_open { "w-64" } else { "w-16" }
            )}>
                <button onclick={toggle} class="text-sm text-white mb-4">
                    { if *is_open { "<<" } else { ">>" } }
                </button>

                { if *is_open {
                    html! {
                        <>
                            <div class="mb-4">
                                <label class="block mb-2 font-semibold">{"Upload Scene ZIP"}</label>
                                <input type="file" accept=".zip"
                                    onchange={on_file_change.clone()}
                                    class="text-black bg-white px-2 py-1 rounded" />
                            </div>

                            <div>
                                <h2 class="text-lg font-bold mb-2">{"Scenes"}</h2>
                                <ul class="space-y-2">
                                    { for props.scenes.iter().map(|scene| html! {
                                        <div class="rounded-lg bg-white/10 p-3 shadow hover:bg-white/20 transition-all">
                                            <div class="text-sm font-medium">{ &scene.name }</div>
                                            <div class="text-xs text-gray-300 truncate">{ &scene.id.to_string() }</div>
                                            <button class="mt-2 text-xs text-blue-300 hover:underline">{"Load Scene"}</button>
                                        </div>
                                    })}
                                </ul>
                            </div>
                        </>
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
    
     */
}
