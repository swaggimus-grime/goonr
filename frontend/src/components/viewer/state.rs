use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use futures_util::TryStreamExt;
use glam::{Quat, Vec2, Vec3};
use gloo::render::request_animation_frame;
use gloo_console::{error, log, warn};
use wasm_bindgen::prelude::Closure;
use web_sys::{window, HtmlCanvasElement};
use web_cmn::pipeline::WiredPipelineMessage;
use web_cmn::splats::RawSplats;
use crate::error::{FrontendError, Result};
use std::borrow::BorrowMut;
use std::ops::DerefMut;
use websplat::camera::Camera;
use websplat::Splatter;

pub struct ViewerState {
    pub(crate) ctx: websplat::Context,
    splatter: Splatter,
    camera: websplat::camera::Camera,
    mouse_down: bool,
    last_mouse_pos: Option<(f32, f32)>,
}

impl ViewerState {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self> {
        let (width, height) = (canvas.width(), canvas.height());
        match websplat::Context::new(canvas, width, height).await {
            Ok(ctx) => {
                let raw = RawSplats {
                    means: vec![ -0.5, 0.0, 0.0,   0.5, 0.0, 0.0 ],
                    rotation: vec![
                        // identity for both
                        1.0,0.0,0.0,  0.0,1.0,0.0,  0.0,0.0,1.0,
                        1.0,0.0,0.0,  0.0,1.0,0.0,  0.0,0.0,1.0,
                    ],
                    log_scales: vec![ (-0.5f32).ln(), (-0.5f32).ln(), (-0.5f32).ln(),
                                      (-0.3f32).ln(), (-0.3f32).ln(), (-0.3f32).ln() ],
                    sh_coeffs: vec![1.0f32; 54], // 27 per splat
                    sh_coeffs_dims: [2, 3, 9],
                    raw_opacity: vec![1.0, 0.8],
                };

                let splatter = Splatter::new(&ctx);
                let camera = Camera::new(width as f32 / height as f32);

                Ok(Self {
                    ctx,
                    splatter,
                    camera,
                    mouse_down: false,
                    last_mouse_pos: None,
                })
            }
            Err(err) => {
                Err(FrontendError::ViewerError(err))
            }
        }
    }


    pub fn on_pipeline_msg(&mut self, msg: WiredPipelineMessage) {
        match msg {
            WiredPipelineMessage::TrainStep(mut splats) => {
                self.splatter.set_splats(&self.ctx, &splats);
            }
            _ => {
                let msg = format!("Unhandled pipeline message: {:?}", msg);
                warn!(msg);
            }
        }
    }

    pub fn handle_mouse_down(&mut self) {
        self.mouse_down = true;
    }

    pub fn handle_mouse_up(&mut self) {
        self.mouse_down = false;
        self.last_mouse_pos = None;
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        if !self.mouse_down {
            return;
        }

        if let Some((last_x, last_y)) = self.last_mouse_pos {
            let dx = x - last_x;
            let dy = y - last_y;
            self.camera.orbit(Vec2::new(dx, dy));
        }

        self.last_mouse_pos = Some((x, y));
    }

    pub fn handle_mouse_wheel(&mut self, zoom: f32) {
        // zoom is usually in "scroll delta" units
        self.camera.zoom(-zoom);
    }

    pub fn render(&mut self) -> Result<()> {
        self.splatter.render(&self.ctx, &self.camera);
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.ctx.resize(width, height);
    }
}
