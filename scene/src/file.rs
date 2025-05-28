use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use tokio::fs::File;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader};
use crate::scene::{Camera, CameraModel, Image, Point3D};

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

pub enum Parser {
    Points(Box<PointsParser>),
    Images(Box<ImagesParser>),
    Cameras(Box<CamerasParser>),
}

pub struct PointsParser;
pub struct ImagesParser;
pub struct CamerasParser;

type ParseResult = Pin<Box<dyn Future<Output = io::Result<InputData>> + Send>>;

pub trait Parseable: Send + Sync {
    fn parse_bin(&self, reader: BufReader<File>) -> ParseResult;
    fn parse_txt(&self, reader: BufReader<File>) -> ParseResult;
}

fn parse<T: std::str::FromStr>(s: &str) -> io::Result<T> {
    s.parse()
        .map_err(|_e| io::Error::new(io::ErrorKind::InvalidData, "Parse error"))
}

impl Parseable for ImagesParser {
    fn parse_bin(&self, mut reader: BufReader<File>) -> ParseResult {
        Box::pin(async move {
            let mut images = HashMap::new();
            let num_images = reader.read_u64_le().await?;

            for _ in 0..num_images {
                let image_id = reader.read_i32_le().await?;

                let [w, x, y, z] = [
                    reader.read_f64_le().await? as f32,
                    reader.read_f64_le().await? as f32,
                    reader.read_f64_le().await? as f32,
                    reader.read_f64_le().await? as f32,
                ];
                let quat = glam::quat(x, y, z, w);

                let tvec = glam::vec3(
                    reader.read_f64_le().await? as f32,
                    reader.read_f64_le().await? as f32,
                    reader.read_f64_le().await? as f32,
                );

                let camera_id = reader.read_i32_le().await?;
                let mut name_bytes = Vec::new();
                reader.read_until(b'\0', &mut name_bytes).await?;

                let name = std::str::from_utf8(&name_bytes[..name_bytes.len() - 1])
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                    .to_owned();

                let num_points2d = reader.read_u64_le().await?;
                let mut xys = Vec::with_capacity(num_points2d as usize);
                let mut point3d_ids = Vec::with_capacity(num_points2d as usize);

                for _ in 0..num_points2d {
                    xys.push(glam::Vec2::new(
                        reader.read_f64_le().await? as f32,
                        reader.read_f64_le().await? as f32,
                    ));
                    point3d_ids.push(reader.read_i64().await?);
                }

                images.insert(
                    image_id,
                    Image {
                        quat,
                        tvec,
                        camera_id,
                        name,
                        xys,
                        point3d_ids,
                    },
                );
            }

            Ok(InputData::Images(images))
        })
    }
    
    fn parse_txt(&self, mut reader: BufReader<File>) -> ParseResult {
        Box::pin(async move {
            let mut images = HashMap::new();
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();
    
            let mut img_data = true;
    
            loop {
                line.clear();
                if buf_reader.read_line(&mut line).await? == 0 {
                    break;
                }
    
                if !line.is_empty() && !line.starts_with('#') {
                    let elems: Vec<&str> = line.split_whitespace().collect();
                    let id: i32 = parse(elems[0])?;
    
                    let [w, x, y, z] = [
                        parse(elems[1])?,
                        parse(elems[2])?,
                        parse(elems[3])?,
                        parse(elems[4])?,
                    ];
                    let quat = glam::quat(x, y, z, w);
                    let tvec = glam::vec3(parse(elems[5])?, parse(elems[6])?, parse(elems[7])?);
                    let camera_id: i32 = parse(elems[8])?;
                    let name = elems[9].to_owned();
    
                    line.clear();
                    buf_reader.read_line(&mut line).await?;
                    let elems: Vec<&str> = line.split_whitespace().collect();
                    let mut xys = Vec::new();
                    let mut point3d_ids = Vec::new();
    
                    for chunk in elems.chunks(3) {
                        xys.push(glam::vec2(parse(chunk[0])?, parse(chunk[1])?));
                        point3d_ids.push(parse(chunk[2])?);
                    }
    
                    images.insert(
                        id,
                        Image {
                            quat,
                            tvec,
                            camera_id,
                            name,
                            xys,
                            point3d_ids,
                        },
                    );
                }
            }
    
            Ok(InputData::Images(images))
        })
    }
}

impl Parseable for PointsParser {
    fn parse_bin(&self, mut reader: BufReader<File>) -> ParseResult {
        Box::pin(async move {
            let mut points3d = HashMap::new();
            let num_points = reader.read_u64_le().await?;
    
            for _ in 0..num_points {
                let point3d_id = reader.read_i64().await?;
                let xyz = glam::Vec3::new(
                    reader.read_f64_le().await? as f32,
                    reader.read_f64_le().await? as f32,
                    reader.read_f64_le().await? as f32,
                );
                let rgb = [
                    reader.read_u8().await?,
                    reader.read_u8().await?,
                    reader.read_u8().await?,
                ];
                let error = reader.read_f64_le().await?;
    
                let track_length = reader.read_u64_le().await?;
                let mut image_ids = Vec::with_capacity(track_length as usize);
                let mut point2d_idxs = Vec::with_capacity(track_length as usize);
    
                for _ in 0..track_length {
                    image_ids.push(reader.read_i32_le().await?);
                    point2d_idxs.push(reader.read_i32_le().await?);
                }
    
                points3d.insert(
                    point3d_id,
                    Point3D {
                        xyz,
                        rgb,
                        error,
                        image_ids,
                        point2d_idxs,
                    },
                );
            }
    
            Ok(InputData::Points3D(points3d))
        })
    }

    fn parse_txt(&self, mut reader: BufReader<File>) -> ParseResult {
        Box::pin(async move {
            let mut points3d = HashMap::new();
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();
    
            while buf_reader.read_line(&mut line).await? > 0 {
                if line.starts_with('#') {
                    line.clear();
                    continue;
                }
    
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 8 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid point3D data",
                    ));
                }
    
                let id: i64 = parse(parts[0])?;
                let xyz = glam::Vec3::new(parse(parts[1])?, parse(parts[2])?, parse(parts[3])?);
                let rgb = [
                    parse::<u8>(parts[4])?,
                    parse::<u8>(parts[5])?,
                    parse::<u8>(parts[6])?,
                ];
                let error: f64 = parse(parts[7])?;
    
                let mut image_ids = Vec::new();
                let mut point2d_idxs = Vec::new();
    
                for chunk in parts[8..].chunks(2) {
                    if chunk.len() < 2 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid point3D track data",
                        ));
                    }
                    image_ids.push(parse(chunk[0])?);
                    point2d_idxs.push(parse(chunk[1])?);
                }
    
                points3d.insert(
                    id,
                    Point3D {
                        xyz,
                        rgb,
                        error,
                        image_ids,
                        point2d_idxs,
                    },
                );
                line.clear();
            }
    
            Ok(InputData::Points3D(points3d))
        })
    }
}

impl Parseable for CamerasParser {
    fn parse_bin(&self, mut reader: BufReader<File>) -> ParseResult {
        Box::pin(async move {
            let mut cameras = HashMap::new();
            let num_cameras = reader.read_u64_le().await?;
    
            for _ in 0..num_cameras {
                let camera_id = reader.read_i32_le().await?;
                let model_id = reader.read_i32_le().await?;
                let width = reader.read_u64_le().await?;
                let height = reader.read_u64_le().await?;
    
                let model = CameraModel::from_id(model_id)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid camera model"))?;
    
                let num_params = model.num_params();
                let mut params = Vec::with_capacity(num_params);
                for _ in 0..num_params {
                    params.push(reader.read_f64_le().await?);
                }
    
                cameras.insert(
                    camera_id,
                    Camera {
                        id: camera_id,
                        model,
                        width,
                        height,
                        params,
                    },
                );
            }
    
            Ok(InputData::Cameras(cameras))
        })
    }

    fn parse_txt(&self, mut reader: BufReader<File>) -> ParseResult {
        Box::pin(async move {
            let mut cameras = HashMap::new();
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let mut line = String::new();

            while buf_reader.read_line(&mut line).await? > 0 {
                if line.starts_with('#') {
                    line.clear();
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 4 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid camera data",
                    ));
                }

                let id = parse(parts[0])?;
                let model = CameraModel::from_name(parts[1])
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid camera model"))?;

                let width = parse(parts[2])?;
                let height = parse(parts[3])?;
                let params: Vec<f64> = parts[4..]
                    .iter()
                    .map(|&s| parse(s))
                    .collect::<Result<_, _>>()?;

                if params.len() != model.num_params() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid number of camera parameters",
                    ));
                }

                cameras.insert(
                    id,
                    Camera {
                        id,
                        model,
                        width,
                        height,
                        params,
                    },
                );
                line.clear();
            }

            Ok(InputData::Cameras(cameras))
        })
    }
}