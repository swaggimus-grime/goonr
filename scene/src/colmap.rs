pub mod input;
mod parser;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io;
use crate::colmap::input::{InputData, InputFile, InputFormat, InputType};
use crate::colmap::parser::{CamerasParser, ImagesParser, Parseable, PointsParser};

pub struct ColmapDir {
    input_files: HashMap<InputType, InputFile>,
}

impl ColmapDir {
    pub async fn new(path: &Path) -> io::Result<Self> {
        let paths = from_dir(path).await?;
        let input_files = queries_from_paths(&paths).await?;
        Ok(Self {
            input_files
        })
    }

    pub async fn query(&self, input_type: InputType) -> io::Result<InputData> {
        match &self.input_files.get(&input_type) {
            Some(file) => Ok(file.parse().await?),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "File not found")),
        }
    }
}

async fn from_dir(dir: &Path) -> io::Result<Vec<PathBuf>>  {
    let dir = PathBuf::from(dir);

    let mut paths = Vec::new();
    let mut stack = vec![dir.clone()];

    while let Some(path) = stack.pop() {
        let mut read_dir = tokio::fs::read_dir(&path).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path.clone());
            } else {
                paths.push(path);
            }
        }
    }

    Ok(paths)
}


async fn queries_from_paths(paths: &[PathBuf]) -> io::Result<HashMap<InputType, InputFile>> {
    let mut files = HashMap::new();
    for path in paths {
        if path.extension().is_some() {
            let input_format = match path.extension().unwrap().to_str().unwrap() {
                "txt" => InputFormat::Text,
                "bin" => InputFormat::Binary,
                _ => continue
            };

            let input_type: (InputType, Box<dyn Parseable>) = match path.file_stem().unwrap().to_str().unwrap() {
                "cameras" => (InputType::Cameras, Box::new(CamerasParser)),
                "images" => (InputType::Images, Box::new(ImagesParser)),
                "points3D" => (InputType::Points3D, Box::new(PointsParser)),
                _ => continue
            };

            let file = InputFile::new(input_type.1, input_format, path.clone()).await?;
            files.insert(input_type.0, file);
        }
    }

    Ok(files)
}
