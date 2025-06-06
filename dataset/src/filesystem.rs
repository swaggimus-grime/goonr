use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Clone)]
pub struct Filesystem {
    root: PathBuf,
}

impl Filesystem {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn file_ending_in(&self, target_filename: &str) -> Option<PathBuf> {
        WalkDir::new(&self.root)
            .into_iter()
            .filter_map(Result::ok)
            .find_map(|entry| {
                let path = entry.path();
                let filename = path.file_name()?.to_str()?.to_lowercase();
                (entry.file_type().is_file() && filename == target_filename.to_lowercase())
                    .then(|| path.to_path_buf())
            })
    }
}