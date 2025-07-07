use std::fs::File;
use std::io;
use std::io::{Cursor, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use burn::serde::Serialize;
use glam::UVec2;
use image::{ColorType, DynamicImage, ImageError, ImageReader, ImageDecoder};
use tokio::io::{AsyncRead, AsyncReadExt};
use scene_source::Filesystem;

#[derive(Clone)]
pub struct ImageFile {
    pub path: PathBuf,
    max_res: u32,
    mask_path: Option<PathBuf>,
    size: UVec2,
    color_fmt: ColorType,
    fs: Arc<Filesystem>
}

impl ImageFile {
    pub async fn new(
        fs: Arc<Filesystem>,
        path: &Path,
        mask_path: Option<PathBuf>,
        max_resolution: u32,
    ) -> image::ImageResult<Self> {
        let reader = &mut fs.reader_at_path(path).await?;
        let prelim = preliminary_data(reader).await?;

        Ok(Self {
            path: path.to_path_buf(),
            max_res: max_resolution,
            mask_path,
            size: prelim.0,
            color_fmt: prelim.1,
            fs
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

    pub fn has_alpha(&self) -> bool {
        self.color_fmt.has_alpha() || self.is_masked()
    }

    pub fn width(&self) -> u32 {
        self.dim().x
    }

    pub fn height(&self) -> u32 {
        self.dim().y
    }

    pub fn is_masked(&self) -> bool {
        self.mask_path.is_some()
    }

    pub fn aspect_ratio(&self) -> f32 {
        let dim = self.dim();
        dim.x as f32 / dim.y as f32
    }

    pub async fn load(&self) -> image::ImageResult<DynamicImage> {
        let mut img_bytes = vec![];
        self.fs
            .reader_at_path(&self.path)
            .await?
            .read_to_end(&mut img_bytes)
            .await?;
        let mut img = image::load_from_memory(&img_bytes)?;

        // Copy over mask.
        // TODO: Interleave this work better & speed things up here.
        if let Some(mask_path) = &self.mask_path {
            // Add in alpha channel if needed to the image to copy the mask into.
            let mut masked_img = img.into_rgba8();
            let mut mask_bytes = vec![];
            self.fs
                .reader_at_path(mask_path)
                .await?
                .read_to_end(&mut mask_bytes)
                .await?;
            let mask_img = image::load_from_memory(&mask_bytes)?;
            if mask_img.color().has_alpha() {
                let mask_img = mask_img.into_rgba8();
                for (pixel, mask_pixel) in masked_img.pixels_mut().zip(mask_img.pixels()) {
                    pixel[3] = mask_pixel[3];
                }
            } else {
                let mask_img = mask_img.into_rgb8();
                for (pixel, mask_pixel) in masked_img.pixels_mut().zip(mask_img.pixels()) {
                    pixel[3] = mask_pixel[0];
                }
            }
            img = masked_img.into();
        }
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

async fn preliminary_data<R>(reader: &mut R) -> std::io::Result<(UVec2, ColorType)>
    where R: AsyncRead + Unpin {
    // The maximum size before the entire SOF of JPEG is read is 65548 bytes. Read 20kb to start, and grow if needed. More exotic image formats
    // might need even more data, so loop below will keep reading until we can figure out the dimensions
    // of the image.
    let mut temp_buf = vec![0; 16387];

    let mut n = 0;
    loop {
        let read = reader.read_exact(&mut temp_buf[n..]).await?;

        if read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Reached end of file while trying to decode image format",
            ));
        }

        n += read;

        // Try to decode with what we have (nb, no copying happens here).
        if let Ok(decoder) = ImageReader::new(Cursor::new(&temp_buf[..n]))
            .with_guessed_format()?
            .into_decoder()
        {
            return Ok((decoder.dimensions().into(), decoder.color_type()));
        }
        // Try reading up to double the size.
        temp_buf.resize(temp_buf.len() * 2, 0);
    }
}