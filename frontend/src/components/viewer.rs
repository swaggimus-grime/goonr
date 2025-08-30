mod state;

use std::cell::RefCell;
use std::rc::Rc;

use futures_util::StreamExt;
use gloo::render::{request_animation_frame, AnimationFrame};
use gloo::utils::window;
use gloo_console::{error, log};
use gloo_net::websocket::{futures::WebSocket, Message};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, MouseEvent};
use yew::prelude::*;

use web_cmn::pipeline::WiredPipelineMessage;
use super::viewer::state::ViewerState;

pub enum Msg {
    SetViewerState(ViewerState),
    RenderFrame(f64),
    MouseDown,
    MouseUp,
    MouseMove(f32, f32),
    MouseWheel(f32),
    StartTraining,
    TrainingMsg(WiredPipelineMessage),
    TrainingDone,
    Resize,
}

#[derive(Properties, PartialEq, Clone)]
pub struct ViewerProps {
    pub scene_name: String,
}

pub struct Viewer {
    canvas_ref: NodeRef,
    viewer_state: Option<ViewerState>,
    training: bool,
    raf_handle: Option<AnimationFrame>,
}

impl Component for Viewer {
    type Message = Msg;
    type Properties = ViewerProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas_ref: NodeRef::default(),
            viewer_state: None,
            training: false,
            raf_handle: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // async init ViewerState from the mounted canvas
            let canvas_ref = self.canvas_ref.clone();
            let link = ctx.link().clone();

            spawn_local(async move {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    match ViewerState::new(canvas).await {
                        Ok(viewer_state) => link.send_message(Msg::SetViewerState(viewer_state)),
                        Err(err) => error!(format!("Failed to init ViewerState: {:?}", err)),
                    }
                }
            });

            // window resize listener -> Msg::Resize
            let link = ctx.link().clone();
            let closure = Closure::wrap(Box::new(move || {
                link.send_message(Msg::Resize);
            }) as Box<dyn Fn()>);

            window()
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetViewerState(state) => {
                self.viewer_state = Some(state);

                // kick off RAF loop
                let link = ctx.link().clone();
                let handle = request_animation_frame(move |ts| {
                    link.send_message(Msg::RenderFrame(ts));
                });
                self.raf_handle = Some(handle);

                true
            }

            Msg::RenderFrame(_) => {
                if let Some(state) = self.viewer_state.as_mut() {
                    if let Err(e) = state.render() {
                        error!(format!("Render error: {:?}", e));
                    }
                }
                // schedule next frame
                let link = ctx.link().clone();
                let handle = request_animation_frame(move |ts| {
                    link.send_message(Msg::RenderFrame(ts));
                });
                self.raf_handle = Some(handle);
                false
            }

            Msg::MouseDown => {
                if let Some(state) = self.viewer_state.as_mut() {
                    state.handle_mouse_down();
                }
                false
            }
            Msg::MouseUp => {
                if let Some(state) = self.viewer_state.as_mut() {
                    state.handle_mouse_up();
                }
                false
            }
            Msg::MouseMove(x, y) => {
                if let Some(state) = self.viewer_state.as_mut() {
                    state.handle_mouse_move(x, y);
                }
                false
            }
            Msg::MouseWheel(delta) => {
                if let Some(state) = self.viewer_state.as_mut() {
                    state.handle_mouse_wheel(delta);
                }
                false
            }

            // --- WEBSOCKET: Start / Read / Done ---
            Msg::StartTraining => {
                let scene_name = ctx.props().scene_name.clone();
                let link = ctx.link().clone();
                self.training = true;

                spawn_local(async move {
                    let ws_url = format!("ws://localhost:3000/train/{scene_name}");
                    match WebSocket::open(&ws_url) {
                        Ok(ws) => {
                            let (_write, mut read) = ws.split();
                            log!(format!("Training connected: {}", ws_url));

                            while let Some(msg) = read.next().await {
                                match msg {
                                    Ok(Message::Text(json)) => {
                                        match serde_json::from_str::<WiredPipelineMessage>(&json) {
                                            Ok(p) => link.send_message(Msg::TrainingMsg(p)),
                                            Err(e) => error!(format!("Bad JSON message: {:?}", e)),
                                        }
                                    }
                                    Ok(Message::Bytes(bytes)) => {
                                        // in case backend uses binary; try bincode first then JSON fallback
                                        if let Ok(p) = bincode::deserialize::<WiredPipelineMessage>(&bytes) {
                                            link.send_message(Msg::TrainingMsg(p));
                                        } else if let Ok(txt) = String::from_utf8(bytes.clone()) {
                                            if let Ok(p) = serde_json::from_str::<WiredPipelineMessage>(&txt) {
                                                link.send_message(Msg::TrainingMsg(p));
                                            } else {
                                                error!("Unknown binary payload for TrainingMsg");
                                            }
                                        }
                                    }
                                    Ok(_) => { /* ignore pings/close */ }
                                    Err(e) => {
                                        error!(format!("WebSocket error: {:?}", e));
                                        break;
                                    }
                                }
                            }

                            link.send_message(Msg::TrainingDone);
                        }
                        Err(e) => {
                            error!(format!("Failed to open WebSocket: {:?}", e));
                            link.send_message(Msg::TrainingDone);
                        }
                    }
                });

                true // re-render to update button disabled/label
            }

            Msg::TrainingMsg(pipeline_msg) => {
                if let Some(state) = self.viewer_state.as_mut() {
                    state.on_pipeline_msg(pipeline_msg);
                }
                false
            }

            Msg::TrainingDone => {
                self.training = false;
                true
            }

            Msg::Resize => {
                if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
                    let width = canvas.client_width() as u32;
                    let height = canvas.client_height() as u32;
                    canvas.set_width(width);
                    canvas.set_height(height);

                    if let Some(state) = self.viewer_state.as_mut() {
                        state.resize(width, height);
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onmousedown = ctx.link().callback(|_: MouseEvent| Msg::MouseDown);
        let onmouseup   = ctx.link().callback(|_: MouseEvent| Msg::MouseUp);
        // use on-canvas coords; switch to movement_x/y if your camera expects deltas
        let onmousemove = ctx.link().callback(|e: MouseEvent| {
            Msg::MouseMove(e.offset_x() as f32, e.offset_y() as f32)
        });
        let onwheel = ctx.link().callback(|e: WheelEvent| {
            // prevent scrolling the page
            e.prevent_default();
            Msg::MouseWheel(e.delta_y() as f32)
        });

        html! {
            <div class="relative flex-1 min-h-0 z-0">
                <canvas
                    ref={self.canvas_ref.clone()}
                    width="1920"
                    height="1080"
                    {onmousedown}
                    {onmouseup}
                    {onmousemove}
                    {onwheel}
                    style="cursor: grab;"
                    class="w-full h-full block"
                />
                <div class="absolute bottom-4 left-4 z-10">
                    <button
                        onclick={ctx.link().callback(|_| Msg::StartTraining)}
                        disabled={self.training}
                        class="bg-blue-600 text-white px-4 py-2 rounded shadow hover:bg-blue-700 disabled:opacity-50"
                    >
                        { if self.training { "Training…" } else { "Start Training" } }
                    </button>
                </div>
            </div>
        }
    }
}
