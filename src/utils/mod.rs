mod pwsh;
mod tmpdir;

use crate::error::ScoopieError;
use std::{
    fs::{remove_dir_all, remove_file, DirBuilder},
    path::PathBuf,
};

pub use pwsh::Pwsh;
pub use tmpdir::TempDir;

#[macro_export]
macro_rules! comptime_regex {
    ($pattern:expr) => {
        regex_lite::Regex::new($pattern).expect(&format!("Invalid Regex Pattern: {}", $pattern))
    };
}

pub trait Remove {
    type Error;
    fn rm(&self) -> Result<(), Self::Error>;
}

impl Remove for PathBuf {
    type Error = ScoopieError;

    fn rm(&self) -> Result<(), Self::Error> {
        match self.is_file() {
            true => remove_file(self),
            false => remove_dir_all(&self),
        }
        .map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => ScoopieError::FileNotExist(self.to_path_buf()),
            std::io::ErrorKind::PermissionDenied => ScoopieError::PermissionDenied,
            _ => ScoopieError::Unknown,
        })
    }
}

pub trait CreateDir {
    type Error;
    fn create(path: &Self) -> Result<(), Self::Error>;
}

impl CreateDir for PathBuf {
    type Error = ScoopieError;

    fn create(path: &Self) -> Result<(), Self::Error> {
        DirBuilder::new()
            .recursive(true)
            .create(&path)
            .map_err(|_| ScoopieError::FailedToMkdir(path.to_path_buf()))
    }
}

pub trait Absolute {
    type Error;
    fn absolute(&self) -> Result<PathBuf, Self::Error>;
}

impl Absolute for PathBuf {
    type Error = ScoopieError;

    fn absolute(&self) -> Result<PathBuf, Self::Error> {
        let absolute_path = self
            .canonicalize()
            .map_err(|_| ScoopieError::AbsoultePathResolve)?;
        let absolute_path_str = absolute_path.to_string_lossy().to_string();

        // Remove the `\\?\` prefix from the absolute path string
        Ok(PathBuf::from(
            match absolute_path_str.starts_with("\\\\?\\") {
                true => absolute_path_str[4..].to_string(),
                false => absolute_path_str,
            },
        ))
    }
}
