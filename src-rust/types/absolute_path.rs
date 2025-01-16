use std::{
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsolutePath(PathBuf);

impl AbsolutePath {
    pub fn from(path: &Path, base: Option<&PathBuf>) -> Result<Self> {
        if path.is_absolute() {
            return Ok(Self(path.to_path_buf()));
        }
        match base {
            Some(cwd) => Ok(Self(
                cwd.join(path)
                    .canonicalize()
                    .context("can't canonicalize path")?,
            )),
            None => {
                let cwd = std::env::current_dir()?;
                Ok(Self(
                    cwd.as_path()
                        .join(path)
                        .canonicalize()
                        .context("can't canonicalize path")?,
                ))
            }
        }
    }

    pub fn to_string_lossy(&self) -> String {
        self.0.to_string_lossy().to_string()
    }

    pub fn to_relative(&self, base: Option<&AbsolutePath>) -> Result<PathBuf> {
        match base {
            Some(base) => Ok(self
                .0
                .strip_prefix(base.as_ref())
                .context(format!(
                    "can't strip `{}` from `{}`",
                    base.as_ref().display(),
                    self.0.display()
                ))?
                .to_path_buf()),
            None => {
                let cwd = std::env::current_dir()?;
                Ok(self
                    .0
                    .strip_prefix(cwd.as_path())
                    .context(format!("can't strip cwd from `{}`", self.0.display()))?
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
