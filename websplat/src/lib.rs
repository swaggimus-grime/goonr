use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use wgpu::{Instance, Surface, Adapter, Device, Queue, SurfaceConfiguration};

pub mod camera;
pub mod renderer;
mod error;
mod splats;
mod splatter;
mod context;
mod quad;

pub use context::Context;
pub use splatter::Splatter;
pub use error::WebSplatError;
