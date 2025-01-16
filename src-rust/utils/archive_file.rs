//! A simple module to deal with archive files.
//!
//! - Use [`zip::ZipArchive`] for all the reading operations with the zip format.
//! - Use the 7zip CLI for all the writing and reading of other non-zip archive formats.

use std::{
    collections::HashMap,
    io::{BufReader, Read, Write},
    path::PathBuf,
    pin::Pin,
    sync::Arc,
    task::Poll,
};

use anyhow::{anyhow, Context};
use axum::body::Bytes;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Timelike, Utc};
use futures_core::Stream;
use memfd_exec::{ChildStdout, MemFdExecutable, Stdio};
use tracing::warn;

use crate::utils::config::SUPPORTED_ARCHIVE_FORMATS;

use super::traits::StringUtils;

const SEVEN_ZIP_BIN: &[u8] = include_bytes!("../../.devcontainer/7zz");

#[derive(Debug, Clone, Eq)]
pub struct ItemInArchive {
    pub path: String,
    pub last_modified: DateTime<Utc>,
    pub size: Option<i64>,
}

impl PartialEq for ItemInArchive {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl PartialOrd for ItemInArchive {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemInArchive {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

pub trait ItemsInArchiveUtils {
    fn contains_nomedia(&self, feature_enabled: bool) -> bool;
    fn contains_image(&self) -> bool;
    fn keep_images(&self, nomedia_support: bool) -> Vec<&ItemInArchive>;
}

impl ItemsInArchiveUtils for Vec<ItemInArchive> {
    /// Check if the list of items in archive contains a `.nomedia` file.
    fn contains_nomedia(&self, feature_enabled: bool) -> bool {
        feature_enabled && self.iter().any(|f| f.path == ".nomedia")
    }

    /// Check if the archive contains at least one image file.
    fn contains_image(&self) -> bool {
        self.iter().any(|f| f.path.has_image_ext())
    }

    /// Remove non-image files, ignore subdirs contain `.nomedia` file
    /// (if the feature is enabled).
    fn keep_images(&self, nomedia_support: bool) -> Vec<&ItemInArchive> {
        if !nomedia_support {
            return self
                .iter()
                .filter(move |i| i.path.has_image_ext())
                .collect::<Vec<_>>();
        }

        if self.iter().any(|i| i.path == ".nomedia") {
            return vec![];
        }

        // a/.nomedia -> ignore + add `a` to ignored_prefixes
        // a/b/foo.jpg -> starts with one of the prefixes -> ignore
        let mut ignored_prefixes = self
            .iter()
            .filter(|i| i.path.ends_with(".nomedia"))
            .filter_map(|i| {
                i.path
                    .clone()
                    .strip_suffix("/.nomedia")
                    .map(|p| p.to_string())
            })
            .collect::<Vec<_>>();
        ignored_prefixes.sort();
        ignored_prefixes.dedup();

        self.iter()
            .filter(move |i| {
                if !i.path.contains('/') {
                    return i.path.has_image_ext();
                }
                if ignored_prefixes.iter().any(|p| i.path.starts_with(p)) {
                    return false;
                }
                i.path.has_image_ext()
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ArchiveFileError {
    #[error("`{0:?}` points to nothing")]
    NotExists(PathBuf),
    #[error("`{0:?}` is not a file")]
    NotAFile(PathBuf),
    #[error("`{0:?}` is not an archive")]
    NotAnArchive(PathBuf),
    #[error("can't spawn 7zz process: {0:?}")]
    CantSpawn7z(anyhow::Error),
    #[error("error from 7zz: {0}")]
    SevenZipError(String),
    #[error("can't wait 7zz process to complete: {0:?}")]
    CantWaitToComplete(anyhow::Error),
    #[error("can't take stdin pipe to write to 7zz input")]
    CantTakeStdinPipe,
    #[error("can't take stdout pipe to read 7zz output")]
    CantTakeStdoutPipe,
    #[error("can't take stderr pipe to read 7zz output")]
    CantTakeStderrPipe,
    #[error("can't write to stdin pipe: {0:?}")]
    CantWriteToStdin(std::io::Error),
    #[error("can't read from stdout pipe to buffer: {0:?}")]
    CantReadStdout(std::io::Error),
    #[error("can't read from stderr pipe to buffer: {0:?}")]
    CantReadStderr(std::io::Error),
    #[error("other error: {0:?}")]
    Other(anyhow::Error),
}

impl From<anyhow::Error> for ArchiveFileError {
    fn from(e: anyhow::Error) -> Self {
        Self::Other(e)
    }
}

impl From<std::io::Error> for ArchiveFileError {
    fn from(e: std::io::Error) -> Self {
        Self::Other(e.into())
    }
}

pub trait ArchiveFile {
    /// Check if the archive is a valid archive and supported.
    fn validate(&self) -> Result<(), ArchiveFileError>;

    /// Compress files in [`paths`] into [`PathBuf`].
    ///
    /// [`paths`]: Vec<PathBuf>
    /// [`target_path`]: PathBuf
    fn _create_zip_file(&self, paths: &[PathBuf]) -> Result<(), ArchiveFileError>
    where
        Self: Sized;

    /// List all files in the archive.
    ///
    /// https://superuser.com/a/1073272
    fn list_files_in_archive(&self) -> Result<Vec<ItemInArchive>, ArchiveFileError>;

    /// Read the content of a specified file in the archive.
    ///
    /// https://superuser.com/a/148501
    fn read_file_from_archive(&self, file_name: impl ToString)
        -> Result<Vec<u8>, ArchiveFileError>;

    /// Upsert a buffer to a specified file in the archive.
    fn upsert_file_to_archive(
        &self,
        file_name: impl ToString,
        content: Arc<Vec<u8>>,
    ) -> Result<(), ArchiveFileError>;

    /// Consume the [`PathBuf`] and returns a [`Stream`]-able object that can be pass to
    /// [`axum::body::Body::from_stream`] to stream the content of a specified
    /// file in the archive directly without extracting the whole file.
    ///
    /// To avoid an additional call to the 7z CLI, the file size is manually
    /// provided, it's just to tell clients what the size of file they get,
    /// not affecting the streaming process.
    fn stream_file_from_archive(
        self,
        file_name: impl ToString,
        filesize: Option<i64>,
    ) -> anyhow::Result<impl Stream<Item = Result<Bytes, ArchiveFileError>>>;
}

impl ArchiveFile for PathBuf {
    fn validate(&self) -> Result<(), ArchiveFileError> {
        if !self.exists() {
            return Err(ArchiveFileError::NotExists(self.clone()));
        }
        if !self.is_file() {
            return Err(ArchiveFileError::NotAFile(self.clone()));
        }
        if !SUPPORTED_ARCHIVE_FORMATS.contains(
            &self
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
                .as_str(),
        ) {
            return Err(ArchiveFileError::NotAnArchive(self.clone()));
        }
        Ok(())
    }

    fn _create_zip_file(&self, items: &[PathBuf]) -> Result<(), ArchiveFileError> {
        if self.exists() {
            return Err(anyhow!("target path \"{}\" already exists", self.display()).into());
        }

        items.iter().try_for_each(|path| {
            if !path.exists() {
                return Err(ArchiveFileError::NotExists(path.clone()));
            }
            Ok(())
        })?;

        // 7z a -tzip DestinyTest.zip destiny1.txt destiny4.txt destiny6.txt
        let mut child = MemFdExecutable::new("7zz", SEVEN_ZIP_BIN)
            .arg("a")
            .arg(format!("{}", self.display()))
            .arg("-tzip")
            .args(items.iter().map(|path| format!("{}", path.display())))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ArchiveFileError::CantSpawn7z(e.into()))?;

        let mut stdout_buf: Vec<u8> = vec![];
        std::io::BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| ArchiveFileError::CantTakeStdoutPipe)?,
        )
        .read_to_end(&mut stdout_buf)
        .map_err(ArchiveFileError::CantReadStdout)?;

        let mut stderr_buf: Vec<u8> = vec![];
        std::io::BufReader::new(
            child
                .stderr
                .take()
                .ok_or_else(|| ArchiveFileError::CantTakeStderrPipe)?,
        )
        .read_to_end(&mut stderr_buf)
        .map_err(ArchiveFileError::CantReadStderr)?;

        if !stderr_buf.is_empty() {
            return Err(ArchiveFileError::SevenZipError(
                String::from_utf8_lossy(&stdout_buf).trim().to_string(),
            ));
        }

        child
            .wait()
            .map_err(|e| ArchiveFileError::CantWaitToComplete(e.into()))?;

        if !self.exists() {
            return Err(ArchiveFileError::Other(anyhow!(
                "process done without error, but target path \"{}\" not exists",
                self.display()
            )));
        }

        Ok(())
    }

    fn list_files_in_archive(&self) -> Result<Vec<ItemInArchive>, ArchiveFileError> {
        self.validate()?;

        let mut child = MemFdExecutable::new("7zz", SEVEN_ZIP_BIN)
            .arg("l")
            .arg(format!("{}", self.display()))
            .arg("-ba")
            .arg("-slt")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ArchiveFileError::CantSpawn7z(e.into()))?;

        let mut stdout_buf: Vec<u8> = vec![];
        std::io::BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| ArchiveFileError::CantTakeStdoutPipe)?,
        )
        .read_to_end(&mut stdout_buf)
        .map_err(ArchiveFileError::CantReadStdout)?;

        let mut stderr_buf: Vec<u8> = vec![];
        std::io::BufReader::new(
            child
                .stderr
                .take()
                .ok_or_else(|| ArchiveFileError::CantTakeStderrPipe)?,
        )
        .read_to_end(&mut stderr_buf)
        .map_err(ArchiveFileError::CantReadStderr)?;

        if !stderr_buf.is_empty() {
            return Err(ArchiveFileError::SevenZipError(
                String::from_utf8_lossy(&stdout_buf).trim().to_string(),
            ));
        }

        let mut files: Vec<ItemInArchive> = String::from_utf8_lossy(&stdout_buf)
            .trim()
            .split("\n\n")
            .filter(|s| !s.is_empty())
            .filter_map(|s| 'scoped: {
                let attributes: HashMap<&str, &str> = s
                    .split("\n")
                    .collect::<Vec<_>>()
                    .iter()
                    .filter_map(|line| {
                        let mut split = line.split(" = ");
                        if let (Some(key), Some(value)) = (split.next(), split.next()) {
                            return Some((key, value));
                        }
                        None
                    })
                    .collect();

                let path = attributes
                    .get("Path")
                    .map(|val| val.trim().to_string())
                    .filter(|val| !val.is_empty())
                    .ok_or_else(|| warn!("can't get path for item {}", self.display()))
                    .ok()?;

                let is_dir = attributes
                    .get("Folder")
                    .map(|val| val.trim() == "+")
                    .unwrap_or_else(|| {
                        attributes
                            .get("Size")
                            .map(|val| val.trim() == "0")
                            .unwrap_or_else(|| {
                                warn!("can't check if {path} in {} is a directory", self.display());
                                true
                            })
                    });
                if is_dir {
                    break 'scoped None;
                }

                let last_modified = attributes
                    .get("Modified")
                    .ok_or_else(|| anyhow!("there should exist a modified date"))
                    .and_then(|val| {
                        NaiveDateTime::parse_from_str(val.trim(), "%Y-%m-%d %H:%M:%S%.f")
                            .context("can't parse modified date")
                    })
                    .and_then(|native_datetime| {
                        Local
                            .from_local_datetime(&native_datetime)
                            .single()
                            .context("can't convert modified date to local datetime")
                    })
                    .map(|local_datetime| local_datetime.to_utc())
                    .map_err(|e| {
                        warn!(
                            "can't get last modified date for {path} in {}: {e:#}",
                            self.display()
                        )
                    })
                    .ok()
                    .map(|d| d.with_nanosecond(0).unwrap_or_default())?;

                let size = attributes
                    .get("Size")
                    .ok_or_else(|| warn!("can't get size for item {path} in {}", self.display()))
                    .and_then(|val| {
                        val.trim().parse::<i64>().map_err(|e| {
                            warn!("can't parse size for {path} in {}: {e:#}", self.display())
                        })
                    })
                    .ok();

                Some(ItemInArchive {
                    path,
                    last_modified,
                    size,
                })
            })
            .collect();
        files.sort();
        files.dedup();

        Ok(files)
    }

    fn read_file_from_archive(
        &self,
        file_name: impl ToString,
    ) -> Result<Vec<u8>, ArchiveFileError> {
        self.validate()?;

        // Read content of specified file to stdout
        // 7zz e -so <input> <file-to-extract>
        let mut child = MemFdExecutable::new("7zz", SEVEN_ZIP_BIN)
            .arg("e")
            .arg(format!("{}", self.display()))
            .arg("-so")
            .arg(file_name.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ArchiveFileError::CantSpawn7z(e.into()))?;

        let mut stdout_buf: Vec<u8> = vec![];
        std::io::BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| ArchiveFileError::CantTakeStdoutPipe)?,
        )
        .read_to_end(&mut stdout_buf)
        .map_err(ArchiveFileError::CantReadStdout)?;

        let mut stderr_buf: Vec<u8> = vec![];
        std::io::BufReader::new(
            child
                .stderr
                .take()
                .ok_or_else(|| ArchiveFileError::CantTakeStderrPipe)?,
        )
        .read_to_end(&mut stderr_buf)
        .map_err(ArchiveFileError::CantReadStderr)?;

        child
            .wait()
            .map_err(|e| ArchiveFileError::CantWaitToComplete(e.into()))?;

        if !stderr_buf.is_empty() {
            return Err(ArchiveFileError::SevenZipError(
                String::from_utf8_lossy(&stderr_buf).trim().to_string(),
            ));
        }

        Ok(stdout_buf)
    }

    fn upsert_file_to_archive(
        &self,
        file_name: impl ToString,
        content: Arc<Vec<u8>>,
    ) -> Result<(), ArchiveFileError> {
        self.validate()?;

        let mut child = MemFdExecutable::new("7zz", SEVEN_ZIP_BIN)
            .arg("u")
            .arg(format!("{}", self.display()))
            .arg(format!("-si{}", file_name.to_string()))
            .arg(file_name.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ArchiveFileError::CantSpawn7z(e.into()))?;
        {
            child
                .stdin
                .take()
                .ok_or_else(|| ArchiveFileError::CantTakeStdinPipe)?
                .write_all((*content).as_ref())
                .map_err(ArchiveFileError::CantWriteToStdin)?;
        }
        let child_output = child
            .wait_with_output()
            .map_err(|e| ArchiveFileError::CantWaitToComplete(e.into()))?;

        if !child_output.stderr.is_empty() {
            return Err(ArchiveFileError::SevenZipError(
                String::from_utf8_lossy(&child_output.stderr)
                    .trim()
                    .to_string(),
            ));
        }

        Ok(())
    }

    fn stream_file_from_archive(
        self,
        file_name: impl ToString,
        filesize: Option<i64>,
    ) -> anyhow::Result<impl Stream<Item = Result<Bytes, ArchiveFileError>>> {
        self.validate()?;

        let mut child = MemFdExecutable::new("7zz", SEVEN_ZIP_BIN)
            .arg("e")
            .arg(format!("{}", self.display()))
            .arg("-so")
            .arg(file_name.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ArchiveFileError::CantSpawn7z(e.into()))?;
        let buf = BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| ArchiveFileError::CantTakeStdoutPipe)?,
        );

        Ok(ArchiveItemStream {
            reader: buf,
            filesize,
        })
    }
}

#[derive(Debug)]
struct ArchiveItemStream {
    reader: BufReader<ChildStdout>,
    filesize: Option<i64>,
}

impl Stream for ArchiveItemStream {
    type Item = Result<Bytes, ArchiveFileError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let _ = cx;
        let mut buf = vec![0; 65536];
        match self.reader.read(&mut buf) {
            Ok(0) => Poll::Ready(None),
            Ok(n) => Poll::Ready(Some(Ok(Bytes::from(buf[..n].to_vec())))),
            Err(e) => Poll::Ready(Some(Err(ArchiveFileError::CantReadStdout(e)))),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            0,
            self.filesize.and_then(|filesize| match filesize {
                size if size <= 0 => None,
                size if size as usize > usize::MAX => None,
                size => Some(size as usize),
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{self, BufReader, Write},
        sync::Arc,
    };

    use chrono::{DateTime, Timelike, Utc};
    use memfd_exec::{MemFdExecutable, Stdio};
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn available_only_on_x86_64_linux() {
        #[cfg(not(target_arch = "x86_64"))]
        {
            assert!(false);
        }
    }

    #[test]
    fn seven_zip_cli_in_path() {
        let mut child = MemFdExecutable::new("7zz", &SEVEN_ZIP_BIN)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("can't spawn 7zz");

        let output = io::read_to_string(&mut BufReader::new(
            child.stdout.take().expect("can't take stdout"),
        ))
        .expect("can't read stdout");
        let err = io::read_to_string(&mut BufReader::new(
            child.stderr.take().expect("can't take stderr"),
        ))
        .expect("can't read stderr");

        assert!(output.contains("7-Zip (z) 24.08 (x64)"));
        assert!(err.is_empty());
    }

    #[test]
    fn create_zip_list_files() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();

        let archive_file = temp_dir.path().join("new.zip");
        archive_file._create_zip_file(&vec![test_file]).unwrap();
        let files = archive_file.list_files_in_archive().unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "test.txt");
    }

    #[test]
    fn read_file() {
        let temp_dir = TempDir::new().unwrap();

        let test_file1 = temp_dir.path().join("test1.txt");
        let mut file1 = File::create(&test_file1).unwrap();
        file1.write_all(b"lorem ipsum").unwrap();
        file1.flush().unwrap();

        let test_file2 = temp_dir.path().join("test2.txt");
        let mut file2 = File::create(&test_file2).unwrap();
        file2.write_all(b"dolor sit amet").unwrap();
        file2.flush().unwrap();

        let archive_file = temp_dir.path().join("new.zip");
        archive_file
            ._create_zip_file(&vec![test_file1, test_file2])
            .unwrap();

        assert_eq!(
            archive_file.read_file_from_archive("test1.txt").unwrap(),
            b"lorem ipsum"
        );
        assert_eq!(
            archive_file.read_file_from_archive("test2.txt").unwrap(),
            b"dolor sit amet"
        );
    }

    #[test]
    fn upsert_file() {
        let temp_dir = TempDir::new().unwrap();

        let temp_file_name = "test.txt";
        File::create(temp_dir.path().join(temp_file_name)).unwrap();

        // create a zip file w/ one empty file
        let filename_1 = "test.txt";
        let archive_file = temp_dir.path().join("new.zip");
        archive_file
            ._create_zip_file(&vec![temp_dir.path().join(temp_file_name)])
            .unwrap();

        assert_eq!(
            archive_file.read_file_from_archive(&filename_1).unwrap(),
            b""
        );
        assert_eq!(archive_file.list_files_in_archive().unwrap().len(), 1);

        // overwrite that empty file
        let content_1 = Arc::new(b"lorem ipsum".to_vec());
        archive_file
            .upsert_file_to_archive(&filename_1, content_1.clone())
            .unwrap();

        assert_eq!(
            archive_file.read_file_from_archive(&filename_1).unwrap(),
            *content_1
        );
        assert_eq!(archive_file.list_files_in_archive().unwrap().len(), 1);

        // add new file
        let filename_2 = "test2.txt";
        let content_2 = Arc::new(b"dolor sit amet".to_vec());
        File::create(temp_dir.path().join(filename_2)).unwrap();
        archive_file
            .upsert_file_to_archive(&filename_2, content_2.clone())
            .unwrap();

        assert_eq!(
            archive_file.read_file_from_archive(&filename_1).unwrap(),
            *content_1
        );
        assert_eq!(
            archive_file.read_file_from_archive(&filename_2).unwrap(),
            *content_2
        );
        assert_eq!(archive_file.list_files_in_archive().unwrap().len(), 2);
    }

    #[test]
    fn modified_date() {
        let temp_dir = TempDir::new().unwrap();

        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();

        let archive_file = temp_dir.path().join("new.zip");
        archive_file
            ._create_zip_file(&vec![test_file.clone()])
            .unwrap();

        let modified_date_in_zip = archive_file.list_files_in_archive().unwrap()[0].last_modified;
        let real_modified_date: DateTime<Utc> = File::open(&test_file)
            .unwrap()
            .metadata()
            .unwrap()
            .modified()
            .unwrap()
            .into();

        assert_eq!(
            modified_date_in_zip.with_nanosecond(0),
            real_modified_date.with_nanosecond(0)
        );
    }
}
