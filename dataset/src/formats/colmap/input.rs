use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use tokio::io::{AsyncRead, BufReader};
use crate::formats::colmap::camera::Camera;
use crate::formats::colmap::image::Image;
use crate::formats::colmap::parse::{CamerasParser, ImagesParser, Parseable, Parser, PointsParser};
use crate::formats::colmap::point::Point3D;

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
    reader: Box<dyn AsyncRead + Unpin + Send>,
    parser: Box<dyn Parseable>,
    input_format: InputFormat,
}

impl InputFile {
    pub fn new(reader: Box<dyn AsyncRead + Unpin + Send>, input_type: InputType, is_bin: bool) -> InputFile {
        let parser: Box<dyn Parseable> = match input_type {
            InputType::Cameras => Box::new(CamerasParser),
            InputType::Images => Box::new(ImagesParser),
            InputType::Points3D => Box::new(PointsParser)
        };
        
        Self {
            reader,
            input_format: if is_bin { InputFormat::Binary} else { InputFormat::Text },
            parser
        }
    }

    pub async fn parse(self) -> io::Result<InputData> {
        let mut reader = BufReader::new(self.reader);

        match self.input_format {
            InputFormat::Binary => self.parser.parse_bin(reader).await,
            InputFormat::Text => self.parser.parse_txt(reader).await
        }
    }
}