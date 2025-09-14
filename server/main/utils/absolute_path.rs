use std::{
    fmt::{Display, Formatter},
    path::{Path, PathBuf, StripPrefixError},
};

use chrono::{DateTime, Utc};

use crate::utils::pathbuf_utils::{LastModifiedError, PathBufUtils};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsolutePath(PathBuf);

#[derive(Debug, thiserror::Error)]
pub enum AbsolutePathError {
    #[error("can't get current working dir: {0:?}")]
    GetCurrentWorkingDir(std::io::Error),

    #[error("can't canonicalize path: {0:?}")]
    Canonicalize(std::io::Error),

    #[error("can't strip prefix {0} from path {1}: {0:?}")]
    StripPrefix(String, String, StripPrefixError),

    #[error("can't strip cwd from path {0}: {1:?}")]
    StripCwdPrefix(String, StripPrefixError),
}

#[derive(Debug, thiserror::Error)]
pub enum ListItemsInDirectoryError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("absolute path error: {0}")]
    ToAbsolute(#[from] AbsolutePathError),
}

impl AbsolutePath {
    pub fn from(path: &Path, base: Option<&PathBuf>) -> Result<AbsolutePath, AbsolutePathError> {
        if path.is_absolute() {
            return Ok(AbsolutePath(path.to_path_buf()));
        }
        if let Some(cwd) = base {
            Ok(AbsolutePath(
                cwd.join(path)
                    .canonicalize()
                    .map_err(AbsolutePathError::Canonicalize)?,
            ))
        } else {
            let cwd = std::env::current_dir().map_err(AbsolutePathError::GetCurrentWorkingDir)?;
            Ok(AbsolutePath(
                cwd.as_path()
                    .join(path)
                    .canonicalize()
                    .map_err(AbsolutePathError::Canonicalize)?,
            ))
        }
    }

    pub fn to_string_lossy(&self) -> String {
        self.0.to_string_lossy().to_string()
    }

    pub fn to_relative(&self, base: Option<&AbsolutePath>) -> Result<PathBuf, AbsolutePathError> {
        if let Some(base) = base {
            Ok(self
                .0
                .strip_prefix(base.as_ref())
                .map_err(|e| {
                    AbsolutePathError::StripPrefix(
                        base.to_string_lossy(),
                        self.0.to_string_lossy().to_string(),
                        e,
                    )
                })?
                .to_path_buf())
        } else {
            let cwd = std::env::current_dir().map_err(AbsolutePathError::GetCurrentWorkingDir)?;
            Ok(self
                .0
                .strip_prefix(cwd.as_path())
                .map_err(|e| {
                    AbsolutePathError::StripCwdPrefix(self.0.to_string_lossy().to_string(), e)
                })?
                .to_path_buf())
        }
    }

    /// Alias of `.as_ref().metadata()`
    pub fn metadata(&self) -> Result<std::fs::Metadata, std::io::Error> {
        self.0.metadata()
    }

    /// Alias of `.as_ref().last_modified()`
    pub fn last_modified(&self) -> Result<DateTime<Utc>, LastModifiedError> {
        self.0.last_modified()
    }

    /// Alias of `.as_ref().is_file()`
    pub fn is_file(&self) -> bool {
        self.0.is_file()
    }

    /// Alias of `.as_ref().is_dir()`
    pub fn is_dir(&self) -> bool {
        self.0.is_dir()
    }

    /// Alias of `.as_ref().display()`
    pub fn display(&self) -> impl Display + '_ {
        self.0.display()
    }

    /// List files and directories in a directory (non-recursively)
    /// Returns an empty vector if the path is not a directory
    pub fn list_items_in_directory(&self) -> Result<Vec<AbsolutePath>, ListItemsInDirectoryError> {
        let mut files = vec![];
        for entry in self.as_ref().read_dir()? {
            files.push(entry?.path().to_absolute(None)?);
        }
        Ok(files)
    }
}

impl AsRef<PathBuf> for AbsolutePath {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl Display for AbsolutePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

pub trait ToAbsolute<T> {
    fn to_absolute(&self, base: Option<&T>) -> Result<AbsolutePath, AbsolutePathError>;
}

impl ToAbsolute<PathBuf> for PathBuf {
    fn to_absolute(&self, base: Option<&PathBuf>) -> Result<AbsolutePath, AbsolutePathError> {
        AbsolutePath::from(self.as_path(), base)
    }
}

pub trait VecAbsolutePathUtils {
    fn contains_image(&self) -> bool;
}

impl VecAbsolutePathUtils for Vec<AbsolutePath> {
    /// Check if the vector contains at least one image file path.
    fn contains_image(&self) -> bool {
        self.iter()
            .any(|p| p.is_file() && p.as_ref().has_image_ext())
    }
}
