#[cfg(test)]
mod tests {
    use std::fs::File;

    use chrono::{DateTime, Utc};
    use tempfile::tempdir;
    use tracing_test::traced_test;

    use crate::{
        archive_file::{ArchiveFile, ItemInArchive},
        library_processor::dir_entry_guesser::{
            has_pattern_of_a_series::HasPatternOfSeries, ChapterDirOrArchive, ChapterInfo,
        },
        traits::PathBufUtils,
    };

    macro_rules! into_archive_item {
        ($file:expr, $path:expr) => {
            ItemInArchive {
                path: $path.to_string(),
                last_modified: Some(DateTime::<Utc>::from(
                    $file.metadata().unwrap().modified().unwrap(),
                )),
                size: Some($file.metadata().unwrap().len() as i64),
            }
        };
    }

    #[traced_test]
    #[test]
    fn test_has_pattern_of_a_series() {
        let tmp = tempdir().unwrap();
        let png_file = File::create(tmp.path().join("file.png")).unwrap();
        let jpg_file = File::create(tmp.path().join("file.jpg")).unwrap();
        let txt_file = File::create(tmp.path().join("file.txt")).unwrap();
        File::create(tmp.path().join(".nomedia")).unwrap();

        tmp.path()
            .join("inner/123chapter_001.zip")
            .to_path_buf()
            ._create_zip_file(&vec![
                tmp.path().join("file.png"),
                tmp.path().join("file.jpg"),
                tmp.path().join("file.txt"),
            ])
            .unwrap();
        tmp.path()
            .join("inner/123-----CHAPTER    00200.zip")
            .to_path_buf()
            ._create_zip_file(&vec![tmp.path().join("file.png")])
            .unwrap();
        tmp.path()
            .join("inner/contains_no_image.zip")
            .to_path_buf()
            ._create_zip_file(&vec![tmp.path().join("file.txt")])
            .unwrap();
        tmp.path()
            .join("inner/contains_nomedia.zip")
            .to_path_buf()
            ._create_zip_file(&vec![
                tmp.path().join("file.png"),
                tmp.path().join("file.jpg"),
                tmp.path().join(".nomedia"),
            ])
            .unwrap();

        assert_eq!(
            tmp.path()
                .to_path_buf()
                .join("inner")
                .read_dir()
                .unwrap()
                .map(|e| e.unwrap())
                .collect::<Vec<_>>()
                .has_pattern_of_a_series(true, false)
                .unwrap(),
            vec![
                ScannedChapterInfo {
                    volume: 1,
                    path: tmp.path().join("inner/123chapter_001.zip"),
                    dir_or_archive: ScannedChapterType::Archive(vec![
                        into_archive_item!(jpg_file, "file.jpg"),
                        into_archive_item!(png_file, "file.png"),
                        into_archive_item!(txt_file, "file.txt"),
                    ]),
                },
                ScannedChapterInfo {
                    volume: 200,
                    path: tmp.path().join("inner/123-----CHAPTER    00200.zip"),
                    dir_or_archive: ScannedChapterType::Archive(vec![into_archive_item!(
                        png_file, "file.png"
                    ),]),
                }
            ]
        );

        assert_eq!(
            tmp.path()
                .to_path_buf()
                .join("inner")
                .read_dir()
                .unwrap()
                .map(|e| e.unwrap())
                .collect::<Vec<_>>()
                .has_pattern_of_a_series(false, false),
            None
        );

        tmp.path()
            .join("inner2/00100.zip")
            .to_path_buf()
            ._create_zip_file(&vec![tmp.path().join("file.png")])
            .unwrap();
        tmp.path()
            .join("inner2/002.zip")
            .to_path_buf()
            ._create_zip_file(&vec![tmp.path().join("file.png")])
            .unwrap();

        assert_eq!(
            tmp.path()
                .to_path_buf()
                .join("inner2")
                .read_dir()
                .unwrap()
                .map(|e| e.unwrap())
                .collect::<Vec<_>>()
                .has_pattern_of_a_series(false, false),
            Some(vec![
                ScannedChapterInfo {
                    volume: 100,
                    path: tmp.path().join("inner2/00100.zip"),
                    dir_or_archive: ScannedChapterType::Archive(vec![into_archive_item!(
                        png_file, "file.png"
                    ),]),
                },
                ScannedChapterInfo {
                    volume: 2,
                    path: tmp.path().join("inner2/002.zip"),
                    dir_or_archive: ScannedChapterType::Archive(vec![into_archive_item!(
                        png_file, "file.png"
                    ),]),
                }
            ])
        );
    }

    #[test]
    fn test_contains_file() {
        let tmp = tempdir().unwrap();

        std::fs::create_dir(tmp.path().join("inner")).unwrap();
        std::fs::create_dir(tmp.path().join("inner2")).unwrap();
        File::create(tmp.path().join("inner/.nomedia")).unwrap();
        File::create(tmp.path().join("inner2/CategoryInfo.xml")).unwrap();

        assert!(tmp.path().join("inner").contains_category_info_file());
        assert_eq!(tmp.path().join("inner/CategoryInfo.xml").exists(), false);

        assert_eq!(tmp.path().join("inner2").contains_nomedia_file(true), false);
        assert_eq!(
            tmp.path().join("inner2").contains_nomedia_file(false),
            false
        );
        assert!(tmp.path().join("inner2").contains_category_info_file());
    }
}
