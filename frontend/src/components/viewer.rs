mod state;
mod input;

use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use gloo_console::{debug, error, log};
use gloo_net::websocket::{futures::WebSocket, Message};
use stylist::yew::styled_component;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;
use yew::prelude::*;
use futures_util::StreamExt;

use viewer::Context;
use web_cmn::pipeline::WiredPipelineMessage;
use crate::components::viewer::state::ViewerState;

#[derive(Properties, PartialEq)]
pub struct ViewerProps {
    pub scene_name: String,
}

#[styled_component(Viewer)]
pub fn viewer(props: &ViewerProps) -> Html {
    let canvas_ref = use_node_ref();
    let training = use_state(|| false);
    let state = use_mut_ref(|| None::<ViewerState>);

    {
        let canvas_ref = canvas_ref.clone();
        let state = state.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    *state.borrow_mut() = Some(ViewerState::new(canvas).await.unwrap());
                }
            });
            || ()
        });
    }

    let on_start_training = {
        let scene_name = props.scene_name.clone();
        let state = state.clone();
        let training = training.clone();

        Callback::from(move |_| {
            let scene_name = scene_name.clone();
            let training = training.clone();
            let mut state = state.clone();

            spawn_local(async move {
                let ws_url = format!("ws://localhost:3000/train/{scene_name}");
                match WebSocket::open(&ws_url) {
                    Ok(ws) => {
                        training.set(true);
                        let (_, mut read) = ws.split();

                        log!("Entering training loop");
                        while let Some(msg) = read.next().await {
                            log!("New message");
                            match msg {
                                Ok(Message::Text(json)) => {
                                    match serde_json::from_str::<WiredPipelineMessage>(&json) {
                                        Ok(pipeline_msg) => {
                                            if let Some(state) = state.borrow_mut().deref_mut() {
                                                state.on_pipeline_msg(pipeline_msg);
                                            }
                                        }
                                        Err(err) => {
                                            error!(format!("Failed to deserialize message: {:?}", err));
                                        }
                                    }
                                }
                                Ok(_) => {
                                    log!("Received non-text WebSocket message");
                                }
                                Err(err) => {
                                    error!(format!("WebSocket error: {:?}", err));
                                    training.set(false);
                                    break;
                                }
                            }
                        }

                        log!("WebSocket closed");
                        training.set(false);
                    }
                    Err(err) => {
                        error!("Failed to open WebSocket: {err:?}");
                    }
                }
            });
        })
    };

    html! {
        <div class="relative flex-1 min-h-0 z-0">
            <canvas
                ref={canvas_ref}
                class="w-full h-full block"
            />
            <div class="absolute bottom-4 left-4 z-10">
                <button
                    onclick={on_start_training}
                    disabled={*training}
                    class="bg-blue-600 text-white px-4 py-2 rounded shadow hover:bg-blue-700 disabled:opacity-50"
                >
                    { if *training { "Training…" } else { "Start Training" } }
                </button>
            </div>
        </div>
    }
}
