//! A simple module to deal with archive files.
//!
//! - Use [`zip::ZipArchive`] for all the reading operations with the zip format.
//! - Use the 7zip CLI for all the writing and reading of other non-zip archive formats.

use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use memfd_exec::{MemFdExecutable, Stdio};
use tracing::warn;

const SEVEN_ZIP_BIN: &[u8] = include_bytes!("../../.devcontainer/7zz");

#[derive(Debug)]
pub struct ArchiveFile {
    path: PathBuf,
}

#[derive(Debug, Clone, Eq)]
pub struct ItemInArchive {
    pub path: String,
    pub last_modified: DateTime<Utc>,
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

impl ArchiveFile {
    /// Compress files in [`paths`] to [`target_path`].
    ///
    /// [`paths`]: Vec<PathBuf>
    /// [`target_path`]: PathBuf
    /// https://superuser.com/a/940884
    pub async fn _create(paths: &[PathBuf], target_path: &Path) -> Result<Self> {
        if target_path.exists() {
            return Err(anyhow!(
                "target path \"{}\" already exists",
                target_path.display()
            ));
        }

        paths.iter().try_for_each(|path| {
            if !path.exists() {
                return Err(anyhow!("path \"{}\" not exists", path.display()));
            }
            Ok(())
        })?;

        // 7z a -tzip DestinyTest.zip destiny1.txt destiny4.txt destiny6.txt
        let mut child = MemFdExecutable::new("7zz", SEVEN_ZIP_BIN)
            .arg("a")
            .arg(format!("{}", target_path.display()))
            .arg("-tzip")
            .args(paths.iter().map(|path| format!("{}", path.display())))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("can't spawn 7zz")?;

        let stdout = child.stdout.take().context("can't read stdout")?;
        let stderr = child.stderr.take().context("can't read stderr")?;

        let mut stdout_buf: Vec<u8> = vec![];
        let mut stderr_buf: Vec<u8> = vec![];

        std::io::BufReader::new(stdout)
            .read_to_end(&mut stdout_buf)
            .context("can't read stdout to buffer")?;
        std::io::BufReader::new(stderr)
            .read_to_end(&mut stderr_buf)
            .context("can't read stderr to buffer")?;

        if !stderr_buf.is_empty() {
            return Err(anyhow!(
                "7zz error: {:#}",
                String::from_utf8_lossy(&stdout_buf).trim()
            ));
        }

        child.wait().context("can't wait 7zz process to complete")?;

        if !target_path.exists() {
            return Err(anyhow!(
                "process done without error, but target path \"{}\" not exists",
                target_path.display()
            ));
        }

        Ok(ArchiveFile {
            path: target_path.to_path_buf(),
        })
    }

    /// Create a [`ArchiveFile`] from a path.
    pub fn from(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(anyhow!("archive not exists"));
        }
        if !path.is_file() {
            return Err(anyhow!("archive is not a file"));
        }
        Ok(ArchiveFile { path })
    }

    /// List all files in the archive.
    ///
    /// https://superuser.com/a/1073272
    pub async fn list_files(&mut self) -> Result<Vec<ItemInArchive>> {
        if !self.path.exists() {
            return Err(anyhow!("archive not exists"));
        }

        let mut child = MemFdExecutable::new("7zz", SEVEN_ZIP_BIN)
            .arg("l")
            .arg(format!("{}", self.path.display()))
            .arg("-ba")
            .arg("-slt")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("can't spawn 7zz")?;

        let mut stdout_buf: Vec<u8> = vec![];
        let mut stderr_buf: Vec<u8> = vec![];

        std::io::BufReader::new(child.stdout.take().context("can't read stdout")?)
            .read_to_end(&mut stdout_buf)
            .context("can't read stdout to buffer")?;
        std::io::BufReader::new(child.stderr.take().context("can't read stderr")?)
            .read_to_end(&mut stderr_buf)
            .context("can't read stderr to buffer")?;

        if !stderr_buf.is_empty() {
            return Err(anyhow!(
                "7zz error: {:#}",
                String::from_utf8_lossy(&stdout_buf).trim()
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
                    .filter_map(|line| 'scoped2: {
                        let mut split = line.split(" = ");
                        if let (Some(key), Some(value)) = (split.next(), split.next()) {
                            break 'scoped2 Some((key, value));
                        }
                        None
                    })
                    .collect();

                let path = attributes.get("Path").map(|val| val.trim().to_string());
                let path = match path {
                    Some(path) => path,
                    None => {
                        warn!(
                            "can't get file path for an entry in {}",
                            self.path.display()
                        );
                        break 'scoped None;
                    }
                };

                let is_dir = attributes
                    .get("Folder")
                    .map(|val| val.trim() == "+")
                    .unwrap_or_else(|| {
                        attributes
                            .get("Size")
                            .map(|val| val.trim() == "0")
                            .unwrap_or_else(|| {
                                warn!(
                                    "can't check if {path} in {} is a directory",
                                    self.path.display()
                                );
                                true
                            })
                    });
                if is_dir {
                    return None;
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
                    .map(|local_datetime| local_datetime.to_utc());
                let last_modified = match last_modified {
                    Ok(last_modified) => last_modified,
                    Err(e) => {
                        warn!(
                            "can't get last modified date for file {path} in {}: {e:#}",
                            self.path.display()
                        );
                        break 'scoped None;
                    }
                };

                Some(ItemInArchive {
                    path,
                    last_modified,
                })
            })
            .collect();
        files.sort();
        files.dedup();

        Ok(files)
    }

    /// Read the content of a specified file in the archive.
    ///
    /// https://superuser.com/a/148501
    pub fn read_file(&mut self, file_name: impl ToString) -> Result<Vec<u8>> {
        if !self.path.exists() {
            return Err(anyhow!("archive not exists"));
        }

        // Read content of specified file to stdout
        // 7zz e -so <input> <file-to-extract>
        let mut child = MemFdExecutable::new("7zz", SEVEN_ZIP_BIN)
            .arg("e")
            .arg(format!("{}", self.path.display()))
            .arg("-so")
            .arg(file_name.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("can't spawn 7zz")?;

        let mut stdout_buf: Vec<u8> = vec![];
        let mut stderr_buf: Vec<u8> = vec![];
        std::io::BufReader::new(child.stdout.take().context("can't read stdout")?)
            .read_to_end(&mut stdout_buf)
            .context("can't read stdout to buffer")?;
        std::io::BufReader::new(child.stderr.take().context("can't read stderr")?)
            .read_to_end(&mut stderr_buf)
            .context("can't read stderr to buffer")?;

        if !stderr_buf.is_empty() {
            return Err(anyhow!(
                "7zz error: {:#}",
                String::from_utf8_lossy(&stdout_buf).trim()
            ));
        }

        child.wait().context("can't wait 7zz process to complete")?;

        Ok(stdout_buf)
    }

    /// Write the content of a specified file in the archive to the buffer.
    ///
    /// https://superuser.com/a/148501
    pub async fn write_to_buffer(
        &mut self,
        file_name: impl ToString,
        buf: &mut Vec<u8>,
    ) -> Result<()> {
        if !self.path.exists() {
            return Err(anyhow!("archive not exists"));
        }

        let mut child = MemFdExecutable::new("7zz", SEVEN_ZIP_BIN)
            .arg("e")
            .arg(format!("{}", self.path.display()))
            .arg("-so")
            .arg(file_name.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("can't spawn 7zz")?;

        let mut stderr_buf: Vec<u8> = vec![];
        std::io::BufReader::new(child.stdout.take().context("can't read stdout")?)
            .read_to_end(buf)
            .context("can't read stdout to buffer")?;
        std::io::BufReader::new(child.stderr.take().context("can't read stderr")?)
            .read_to_end(&mut stderr_buf)
            .context("can't read stderr to buffer")?;

        if !stderr_buf.is_empty() {
            return Err(anyhow!(
                "7zz error: {:#}",
                String::from_utf8_lossy(&stderr_buf).trim()
            ));
        }

        child.wait().context("can't wait 7zz process to complete")?;

        Ok(())
    }

    /// Upsert a buffer to a specified file in the archive.
    pub fn upsert_file(&mut self, file_name: impl ToString, content: Arc<Vec<u8>>) -> Result<()> {
        if !self.path.exists() {
            return Err(anyhow!("archive not exists"));
        }

        let mut child = MemFdExecutable::new("7zz", SEVEN_ZIP_BIN)
            .arg("u")
            .arg(format!("{}", self.path.display()))
            .arg(format!("-si{}", file_name.to_string()))
            .arg(file_name.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("can't spawn 7zz")?;
        {
            let mut child_stdin = child.stdin.take().context("can't take stdin")?;
            child_stdin
                .write_all((*content).as_ref())
                .context("can't write to stdin")?;
        }
        let child_output = child
            .wait_with_output()
            .context("can't wait 7zz process to complete")?;

        if !child_output.stderr.is_empty() {
            return Err(anyhow!(
                "7zz error: {:#}",
                String::from_utf8_lossy(&child_output.stderr).trim()
            ));
        }

        Ok(())
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
    use tempdir::TempDir;

    use super::{ArchiveFile, SEVEN_ZIP_BIN};

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

    #[tokio::test]
    async fn create_zip_list_files() {
        let temp_dir = TempDir::new("create-zip-and-list-read-files").unwrap();
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();

        let mut archive_file =
            ArchiveFile::_create(&vec![test_file], &temp_dir.path().join("new.zip"))
                .await
                .unwrap();
        let files = archive_file.list_files().await.unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "test.txt");
    }

    #[tokio::test]
    async fn read_file() {
        let temp_dir = TempDir::new("read-file").unwrap();

        let test_file1 = temp_dir.path().join("test1.txt");
        let mut file1 = File::create(&test_file1).unwrap();
        file1.write_all(b"lorem ipsum").unwrap();
        file1.flush().unwrap();

        let test_file2 = temp_dir.path().join("test2.txt");
        let mut file2 = File::create(&test_file2).unwrap();
        file2.write_all(b"dolor sit amet").unwrap();
        file2.flush().unwrap();

        let mut archive_file = ArchiveFile::_create(
            &vec![test_file1, test_file2],
            &temp_dir.path().join("new.zip"),
        )
        .await
        .unwrap();

        assert_eq!(archive_file.read_file("test1.txt").unwrap(), b"lorem ipsum");
        assert_eq!(
            archive_file.read_file("test2.txt").unwrap(),
            b"dolor sit amet"
        );
    }

    #[tokio::test]
    async fn upsert_file() {
        let temp_dir = TempDir::new("upsert-file").unwrap();

        let temp_file_name = "test.txt";
        File::create(temp_dir.path().join(temp_file_name)).unwrap();

        // create a zip file w/ one empty file
        let filename_1 = "test.txt";
        let mut archive_file = ArchiveFile::_create(
            &vec![temp_dir.path().join(temp_file_name)],
            &temp_dir.path().join("new.zip"),
        )
        .await
        .unwrap();

        assert_eq!(archive_file.read_file(&filename_1).unwrap(), b"");
        assert_eq!(archive_file.list_files().await.unwrap().len(), 1);

        // overwrite that empty file
        let content_1 = Arc::new(b"lorem ipsum".to_vec());
        archive_file
            .upsert_file(&filename_1, content_1.clone())
            .unwrap();

        assert_eq!(archive_file.get_file(&filename_1).unwrap(), *content_1);
        assert_eq!(archive_file.list_files().await.unwrap().len(), 1);

        // add new file
        let filename_2 = "test2.txt";
        let content_2 = Arc::new(b"dolor sit amet".to_vec());
        File::create(temp_dir.path().join(filename_2)).unwrap();
        archive_file
            .upsert_file(&filename_2, content_2.clone())
            .unwrap();

        assert_eq!(archive_file.get_file(&filename_1).unwrap(), *content_1);
        assert_eq!(archive_file.get_file(&filename_2).unwrap(), *content_2);
        assert_eq!(archive_file.list_files().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn modified_date() {
        let temp_dir = TempDir::new("modified-date").unwrap();

        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();

        let mut archive_file =
            ArchiveFile::_create(&vec![test_file.clone()], &temp_dir.path().join("new.zip"))
                .await
                .unwrap();

        let modified_date_in_zip = archive_file.list_files().await.unwrap()[0].last_modified;
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
