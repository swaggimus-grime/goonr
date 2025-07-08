use std::cell::RefCell;
use std::rc::Rc;
use gloo_console::{error, log, warn};
use web_sys::HtmlCanvasElement;
use viewer::Context;
use web_cmn::pipeline::WiredPipelineMessage;
use web_cmn::splats::RawSplats;
use crate::error::{FrontendError, Result};

pub struct ViewerState<'window> {
    splat_queue: Vec<RawSplats>,
    ctx: Context<'window>
}

impl<'window> ViewerState<'window> {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self> {
        match Context::from_html_canvas(canvas).await {
            Ok(ctx) => {
                Ok(Self {
                    splat_queue: Vec::new(),
                    ctx
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
                self.ctx.draw_splats(splats);
            }
            _ => {
                let msg = format!("Unhandled pipeline message: {:?}", msg);
                warn!(msg);
            }
        }
    }
}