use std::fs;
use std::collections::HashMap;
use std::fs::DirEntry;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::task::spawn_blocking;
use zip::ZipArchive;
use crate::scene::file::{CamerasParser, ImagesParser, InputData, Parseable, Parser, PointsParser};

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

pub struct ColmapDir {
    input_files: HashMap<InputType, InputFile>,
}

struct InputFile {
    parser: Box<dyn Parseable>,
    input_format: InputFormat,
    path: PathBuf,
}

impl InputFile {
    async fn new(parser: Box<dyn Parseable>, input_format: InputFormat, path: PathBuf) -> io::Result<Self> {
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

impl ColmapDir {
    pub async fn new(path: &Path) -> io::Result<Self> {
        let paths = traverse_paths(path).await?;

        Ok(Self {
            input_files: queries_from_paths(paths.as_slice()).await?
        })
    }
    
    pub async fn query(&self, input_type: InputType) -> io::Result<InputData> {
        match &self.input_files.get(&input_type) {
            Some(file) => Ok(file.parse().await?),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "File not found")),
        }
    }
}

#[derive(Clone)]
pub struct ZipData {
    data: Arc<Vec<u8>>,
}

impl AsRef<[u8]> for ZipData {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

async fn traverse_paths(path: &Path) -> io::Result<Vec<PathBuf>> {
    if path.is_dir() {
        from_dir(path).await
    } else {
        from_file(path).await
    }
}

async fn from_file(path: &Path) -> io::Result<Vec<PathBuf>> {
    let path = path.to_owned();
    
    spawn_blocking(async move || {
        let file = tokio::fs::File::open(path).await?;
        let mut reader = BufReader::new(file);
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let mut archive = ZipArchive::new(Cursor::new(ZipData {
            data: Arc::new(bytes),
        }))?;

        let mut paths = Vec::new();

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            paths.push(PathBuf::from(file.name()));
        }

        Ok::<Vec<PathBuf>, io::Error>(paths)
    }).await?;
    
    Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
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
                let path = path
                    .strip_prefix(dir.clone())
                    .map_err(|_e| io::ErrorKind::InvalidInput)?
                    .to_path_buf();
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
            
            let input_type: (InputType, Box<dyn Parseable>) = match path.file_name().unwrap().to_str().unwrap() {
                "cameras" => (InputType::Cameras, Box::new(CamerasParser)),
                "images" => (InputType::Images, Box::new(ImagesParser)),
                "points" => (InputType::Points3D, Box::new(PointsParser)),
                _ => continue
            };
            
            let file = InputFile::new(input_type.1, input_format, path.clone()).await?;
            files.insert(input_type.0, file);
        }
    }
    
    Ok(files)
}
