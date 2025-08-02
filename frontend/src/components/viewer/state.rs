use std::cell::RefCell;
use std::rc::Rc;
use futures_util::TryStreamExt;
use glam::{Vec2, Vec3};
use gloo::render::request_animation_frame;
use gloo_console::{error, log, warn};
use wasm_bindgen::prelude::Closure;
use web_sys::{window, HtmlCanvasElement};
use viewer::Context;
use web_cmn::pipeline::WiredPipelineMessage;
use web_cmn::splats::RawSplats;
use crate::components::viewer::input::InputController;
use crate::error::{FrontendError, Result};

pub struct ViewerState<'window> {
    splat_queue: Vec<RawSplats>,
    pub(crate) ctx: Context<'window>,
    pub(crate) renderer: viewer::Renderer,
    input: InputController
}

impl<'window> ViewerState<'window> {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self> {
        match Context::from_html_canvas(canvas.clone()).await {
            Ok(ctx) => {
                let renderer = viewer::Renderer::new(&ctx);
                let input = InputController::new(canvas);

                Ok(Self {
                    splat_queue: Vec::new(),
                    ctx,
                    renderer,
                    input
                })
            }
            Err(err) => {
                Err(FrontendError::ViewerError(err))
            }
        }
    }

    pub fn on_pipeline_msg(&mut self, msg: WiredPipelineMessage) {
        match msg {
            WiredPipelineMessage::TrainStep(splats) => {
                self.ctx.upload_splats(&splats);
            }
            _ => {
                let msg = format!("Unhandled pipeline message: {:?}", msg);
                warn!(msg);
            }
        }
    }

    pub fn update_camera_from_input(&mut self, dt: f32) {
        let input = self.input.state.borrow();

        // Movement vector: X=left/right, Y=up/down, Z=forward/backward
        let mut movement = Vec3::ZERO;
        if input.forward {
            movement.z += 1.0;
        }
        if input.backward {
            movement.z -= 1.0;
        }
        if input.left {
            movement.x -= 1.0;
        }
        if input.right {
            movement.x += 1.0;
        }
        if movement.length_squared() > 0.0 {
            movement = movement.normalize();
        }

        let speed = 0.01; // units per second, adjust as needed
        movement *= speed;

        // Rotation vector: X = horizontal mouse movement, Y = vertical mouse movement
        let rotation = if input.mouse_pressed {
            input.mouse_delta * 0.002 // sensitivity scaling
        } else {
            Vec2::ZERO
        };

        drop(input); // release borrow before mutable borrow

        self.ctx.camera.apply_input(movement, rotation, dt);

        // Clear mouse delta so it doesn't accumulate
        self.input.state.borrow_mut().mouse_delta = Vec2::ZERO;
    }

    pub fn render(&mut self) {
        self.ctx.render_frame(&self.renderer);
    }
}

fn now() -> f64 {
    window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}