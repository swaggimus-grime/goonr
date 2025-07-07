use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Error, Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use path_clean::PathClean;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
use zip::ZipArchive;
use crate::error::Result;
use crate::SceneSourceError;

enum FsNode {
    File(File),
    Dir(Dir),
}

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct Dir {
    pub name: String,
    pub children: Vec<usize>, // references to FsNode in Slab
}

#[derive(Clone)]
enum Container {
    Zip(ZipArchive<Cursor<ZipData>>)
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

/// The virtual filesystem
pub struct Filesystem {
    pub lookup: HashMap<PathKey, PathBuf>,
    container: Container,
}

impl Filesystem {
    pub async fn from_reader(reader: impl AsyncRead + Unpin) -> Result<Filesystem> {
        let mut data = BufReader::new(reader);
        let peek = read_at_most(&mut data, 64).await?;
        let mut reader =
            Box::new(AsyncReadExt::chain(Cursor::new(peek.clone()), data));

        if peek.as_slice().starts_with(b"ply") {
            todo!();
        } else if peek.starts_with(b"PK") {
            let mut bytes = vec![];
            reader.read_to_end(&mut bytes).await?;
            let archive = ZipArchive::new(Cursor::new(ZipData {
                data: Arc::new(bytes),
            }))?;
            let file_names: Vec<_> = archive.file_names().map(PathBuf::from).collect();
            Ok(Self {
                lookup: lookup_from_paths(&file_names),
                container: Container::Zip(archive),
            })
        } else if peek.starts_with(b"<!DOCTYPE html>") {
            todo!();
        } else {
            Err(SceneSourceError::UnknownSource)
        }
    }

    pub fn files_with_extension<'a>(
        &'a self,
        extension: &'a str,
    ) -> impl Iterator<Item = PathBuf> + 'a {
        let extension = extension.to_lowercase();

        self.lookup.values().filter_map(move |path| {
            let ext = path
                .extension()
                .and_then(|ext| ext.to_str())?
                .to_lowercase();
            (ext == extension).then(|| path.clone())
        })
    }

    pub fn files_ending_in<'a>(&'a self, end_path: &'a str) -> impl Iterator<Item = PathBuf> + 'a {
        let end_keyed = PathKey::from_path(Path::new(end_path)).0;

        self.lookup
            .iter()
            .filter(move |kv| kv.0.0.ends_with(&end_keyed))
            .map(|kv| kv.1.clone())
    }

    pub fn files_with_stem<'a>(&'a self, filestem: &'a str) -> impl Iterator<Item = PathBuf> + 'a {
        let filestem = filestem.to_lowercase();
        self.lookup.values().filter_map(move |path| {
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())?
                .to_lowercase();
            (stem == filestem).then(|| path.clone())
        })
    }

    pub async fn reader_at_path(&self, path: &Path) -> io::Result<Box<dyn AsyncRead + Unpin + Send>> {
        let key = PathKey::from_path(path);
        let path = self.lookup.get(&key).ok_or_else(|| {
            Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            )
        })?;

        match &self.container {
            Container::Zip(archive) => {
                let name = path
                    .to_str()
                    .expect("Invalid UTF-8 in zip file")
                    .replace('\\', "/");
                let mut buffer = vec![];
                archive.clone().by_name(&name)?.read_to_end(&mut buffer)?;
                Ok(Box::new(Cursor::new(buffer)))
            }
        }
    }
}

async fn read_at_most<R: AsyncRead + Unpin>(reader: &mut R, limit: usize) -> io::Result<Vec<u8>> {
    let mut buffer = vec![0; limit];
    let bytes_read = reader.read(&mut buffer).await?;
    buffer.truncate(bytes_read);
    Ok(buffer)
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct PathKey(String);

impl PathKey {
    fn from_path(path: &Path) -> Self {
        let key = path
            .clean()
            .to_str()
            .expect("Path is not valid ascii")
            .to_lowercase()
            .replace('\\', "/");
        let key = if key.starts_with('/') {
            key
        } else {
            '/'.to_string() + &key
        };
        Self(key)
    }
}

fn lookup_from_paths(paths: &[PathBuf]) -> HashMap<PathKey, PathBuf> {
    let mut result = HashMap::new();
    for path in paths {
        let path = path.clean();

        // Only consider files with extensions for now. Zip files report directories as paths with no extension (not ending in '/')
        // so can't really differentiate extensionless files from directories. We don't need any files without extensions
        // so just skip them.
        if path.extension().is_some() && !path.components().any(|c| c.as_os_str() == "__MACOSX") {
            let key = PathKey::from_path(&path);
            assert!(
                result.insert(key, path.clone()).is_none(),
                "Duplicate path found: {}. Paths must be unique (case non-sensitive)",
                path.display()
            );
        }
    }
    result
}