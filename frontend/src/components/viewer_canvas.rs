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

#[derive(Properties, PartialEq)]
pub struct ViewerCanvasProps {
    pub scene_name: String,
}

#[styled_component(ViewerCanvas)]
pub fn viewer_canvas(props: &ViewerCanvasProps) -> Html {
    let canvas_ref = use_node_ref();
    let training = use_state(|| false);
    let renderer = use_mut_ref(|| None::<Context>);

    {
        let canvas_ref = canvas_ref.clone();
        let renderer = renderer.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    match Context::from_html_canvas(canvas).await {
                        Ok(ctx) => {
                            *renderer.borrow_mut() = Some(ctx);
                            log!("Renderer initialized");
                        }
                        Err(err) => {
                            error!("Failed to init renderer: {err:?}");
                        }
                    }
                }
            });
            || ()
        });
    }

    let on_start_training = {
        let scene_name = props.scene_name.clone();
        let training = training.clone();

        Callback::from(move |_| {
            let scene_name = scene_name.clone();
            let training = training.clone();

            spawn_local(async move {
                let ws_url = format!("ws://localhost:3000/train/{scene_name}");
                match WebSocket::open(&ws_url) {
                    Ok(ws) => {
                        training.set(true);
                        log!("Training started via WebSocket");

                        let (_, mut read) = ws.split();

                        log!("Entering training loop");
                        while let Some(msg) = read.next().await {
                            log!("New message");
                            match msg {
                                Ok(Message::Text(json)) => {
                                    log!("Attempting to deserialize message");
                                    match serde_json::from_str::<WiredPipelineMessage>(&json) {
                                        Ok(pipeline_msg) => {
                                            let log_msg = format!("Received message: {:#?}", pipeline_msg);
                                            log!(log_msg.as_str());
                                            // TODO: handle PipelineMessage (e.g., update viewer)
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
        <>
            <canvas
                ref={canvas_ref}
                class="absolute top-0 left-0 w-full h-full z-0"
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
        </>
    }
}
