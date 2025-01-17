use std::{
    fmt::{Display, Formatter},
    path::{Path, PathBuf, StripPrefixError},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsolutePath(PathBuf);

#[derive(Debug, thiserror::Error)]
pub enum AbsolutePathErr {
    #[error("can't get current working dir: {0:?}")]
    GetCurrentWorkingDir(std::io::Error),
    #[error("can't canonicalize path: {0:?}")]
    Canonicalize(std::io::Error),

    #[error("can't strip prefix {0} from path {1}: {0:?}")]
    StripPrefix(String, String, StripPrefixError),

    #[error("can't strip cwd from path {0}: {1:?}")]
    StripCwdPrefix(String, StripPrefixError),
}

impl AbsolutePath {
    pub fn from(path: &Path, base: Option<&PathBuf>) -> Result<Self, AbsolutePathErr> {
        if path.is_absolute() {
            return Ok(Self(path.to_path_buf()));
        }
        match base {
            Some(cwd) => Ok(Self(
                cwd.join(path)
                    .canonicalize()
                    .map_err(AbsolutePathErr::Canonicalize)?,
            )),
            None => {
                let cwd = std::env::current_dir().map_err(AbsolutePathErr::GetCurrentWorkingDir)?;
                Ok(Self(
                    cwd.as_path()
                        .join(path)
                        .canonicalize()
                        .map_err(AbsolutePathErr::Canonicalize)?,
                ))
            }
        }
    }

    pub fn to_string_lossy(&self) -> String {
        self.0.to_string_lossy().to_string()
    }

    pub fn to_relative(&self, base: Option<&AbsolutePath>) -> Result<PathBuf, AbsolutePathErr> {
        match base {
            Some(base) => Ok(self
                .0
                .strip_prefix(base.as_ref())
                .map_err(|e| {
                    AbsolutePathErr::StripPrefix(
                        base.to_string_lossy(),
                        self.0.to_string_lossy().to_string(),
                        e,
                    )
                })?
                .to_path_buf()),
            None => {
                let cwd = std::env::current_dir().map_err(AbsolutePathErr::GetCurrentWorkingDir)?;
                Ok(self
                    .0
                    .strip_prefix(cwd.as_path())
                    .map_err(|e| {
                        AbsolutePathErr::StripCwdPrefix(self.0.to_string_lossy().to_string(), e)
                    })?
                    .to_path_buf())
            }
        }
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
    fn to_absolute(&self, base: Option<&T>) -> Result<AbsolutePath, AbsolutePathErr>;
}

impl ToAbsolute<PathBuf> for PathBuf {
    fn to_absolute(&self, base: Option<&PathBuf>) -> Result<AbsolutePath, AbsolutePathErr> {
        AbsolutePath::from(self.as_path(), base)
    }
}
