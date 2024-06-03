use std::path::PathBuf;

pub struct FileSystemChartStore(PathBuf);

impl FileSystemChartStore {
    pub fn new(dir: PathBuf) -> Self {
        Self(dir)
    }
}
