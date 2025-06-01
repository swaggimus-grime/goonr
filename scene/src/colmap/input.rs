use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use crate::camera::Camera;
use crate::colmap::parser::Parseable;
use crate::image::Image;
use crate::point3D::Point3D;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InputType {
    Points3D = 0,
    Images = 1,
    Cameras = 2,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InputFormat {
    Binary = 0,
    Text = 1,
}

#[derive(Debug)]
pub enum InputData {
    Images(HashMap<i32, Image>),
    Points3D(HashMap<i64, Point3D>),
    Cameras(HashMap<i32, Camera>),
}

impl InputData {
    pub fn as_images(self) -> Option<HashMap<i32, Image>> {
        if let InputData::Images(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn as_points(self) -> Option<HashMap<i64, Point3D>> {
        if let InputData::Points3D(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn as_cameras(self) -> Option<HashMap<i32, Camera>> {
        if let InputData::Cameras(map) = self {
            Some(map)
        } else {
            None
        }
    }
}

pub struct InputFile {
    parser: Box<dyn Parseable>,
    input_format: InputFormat,
    path: PathBuf,
}

impl InputFile {
    pub async fn new(parser: Box<dyn Parseable>, input_format: InputFormat, path: PathBuf) -> io::Result<Self> {
        Ok(Self {
            parser,
            input_format,
            path
        })
    }

    pub async fn parse(&self) -> io::Result<InputData> {
        let file = tokio::fs::File::open(self.path.clone()).await?;
        let reader = tokio::io::BufReader::new(file);
        match &self.input_format {
            InputFormat::Binary => Ok(self.parser.parse_bin(reader).await?),
            InputFormat::Text => Ok(self.parser.parse_txt(reader).await?)
        }
    }
}

