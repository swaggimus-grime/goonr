use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use burn::serde::Serialize;
use glam::UVec2;
use image::{ColorType, DynamicImage, ImageError, ImageReader, ImageDecoder};


#[derive(Debug, Clone)]
pub struct ImageFile {
    path: PathBuf,
    max_res: u32,
    size: UVec2,
    color_fmt: ColorType
}

impl ImageFile {
    pub fn new(
        path: &Path,
        max_resolution: u32,
    ) -> image::ImageResult<Self> {
        let prelim = preliminary_data(path)?;

        Ok(Self {
            path: path.to_path_buf(),
            max_res: max_resolution,
            size: prelim.0,
            color_fmt: prelim.1
        })
    }

    pub fn dim(&self) -> glam::UVec2 {
        if self.size.x <= self.max_res && self.size.y <= self.max_res {
            self.size
        } else {
            // Take from image crate, just to be sure logic here matches exactly.
            let wratio = f64::from(self.max_res) / f64::from(self.size.x);
            let hratio = f64::from(self.max_res) / f64::from(self.size.y);
            let ratio = f64::min(wratio, hratio);
            let nw = u64::max((f64::from(self.size.x) * ratio).round() as u64, 1);
            let nh = u64::max((f64::from(self.size.y) * ratio).round() as u64, 1);
            glam::uvec2(nw as u32, nh as u32)
        }
    }

    pub async fn load(&self) -> image::ImageResult<DynamicImage> {
        let mut img_bytes = vec![];
        let mut img = image::load_from_memory(&img_bytes)?;

        if img.width() <= self.max_res && img.height() <= self.max_res {
            return Ok(img);
        }
        Ok(img.resize(
            self.max_res,
            self.max_res,
            image::imageops::FilterType::Triangle,
        ))
    }
}

fn preliminary_data(path: &Path) -> image::ImageResult<(UVec2, ColorType)> {
    let reader = ImageReader::open(path)?;
    if let Ok(decoder) = reader.with_guessed_format()?.into_decoder() {
        Ok((decoder.dimensions().into(), decoder.color_type()))
    } else {
        Err(ImageError::IoError(io::Error::new(ErrorKind::InvalidData, "Ngl I don't know about this image format")))
    }
}