use std::fmt;
use serde::{Deserialize, Serialize};
use crate::splats::RawSplats;

#[derive(Debug, Serialize, Deserialize)]
pub enum WiredPipelineMessage {
    TrainStep(RawSplats),
    Done,
    Error(String),
}