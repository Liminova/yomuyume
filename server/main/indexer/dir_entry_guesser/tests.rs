#[cfg(test)]
mod tests {
    use std::fs::File;

    use chrono::{DateTime, Utc};
    use tempfile::tempdir;
    use tracing_test::traced_test;

    use crate::{
        indexer::dir_entry_guesser::{
            IndexedChapterKind, PartialIndexedChapter, has_series_pattern::HasPatternOfSeries,
        },
        utils::{
            absolute_path::ToAbsolute,
            archive_file::{ArchiveFile, ItemInArchive},
            pathbuf_utils::PathBufUtils,
        },
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
            ._create_zip_file(&[
                tmp.path().join("file.png"),
                tmp.path().join("file.jpg"),
                tmp.path().join("file.txt"),
            ])
            .unwrap();
        tmp.path()
            .join("inner/123-----CHAPTER    00200.zip")
            ._create_zip_file(&[tmp.path().join("file.png")])
            .unwrap();
        tmp.path()
            .join("inner/contains_no_image.zip")
            ._create_zip_file(&[tmp.path().join("file.txt")])
            .unwrap();
        tmp.path()
            .join("inner/contains_nomedia.zip")
            ._create_zip_file(&[
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
                .has_series_pattern(true, false)
                .unwrap(),
            vec![
                PartialIndexedChapter {
                    fallback_vol_num: 1,
                    path: tmp
                        .path()
                        .join("inner/123chapter_001.zip")
                        .to_absolute(None)
                        .unwrap(),
                    kind: IndexedChapterKind::Archive(vec![
                        into_archive_item!(jpg_file, "file.jpg"),
                        into_archive_item!(png_file, "file.png"),
                        into_archive_item!(txt_file, "file.txt"),
                    ]),
                },
                PartialIndexedChapter {
                    fallback_vol_num: 200,
                    path: tmp
                        .path()
                        .join("inner/123-----CHAPTER    00200.zip")
                        .to_absolute(None)
                        .unwrap(),
                    kind: IndexedChapterKind::Archive(vec![into_archive_item!(
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
                .has_series_pattern(false, false),
            None
        );

        tmp.path()
            .join("inner2/00100.zip")
            ._create_zip_file(&[tmp.path().join("file.png")])
            .unwrap();
        tmp.path()
            .join("inner2/002.zip")
            ._create_zip_file(&[tmp.path().join("file.png")])
            .unwrap();

        assert_eq!(
            tmp.path()
                .to_path_buf()
                .join("inner2")
                .read_dir()
                .unwrap()
                .map(|e| e.unwrap())
                .collect::<Vec<_>>()
                .has_series_pattern(false, false),
            Some(vec![
                PartialIndexedChapter {
                    fallback_vol_num: 100,
                    path: tmp
                        .path()
                        .join("inner2/00100.zip")
                        .to_absolute(None)
                        .unwrap(),
                    kind: IndexedChapterKind::Archive(vec![into_archive_item!(
                        png_file, "file.png"
                    ),]),
                },
                PartialIndexedChapter {
                    fallback_vol_num: 2,
                    path: tmp.path().join("inner2/002.zip").to_absolute(None).unwrap(),
                    kind: IndexedChapterKind::Archive(vec![into_archive_item!(
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
        assert!(!tmp.path().join("inner/CategoryInfo.xml").exists());

        assert!(!tmp.path().join("inner2").contains_nomedia_file(true));
        assert!(!tmp.path().join("inner2").contains_nomedia_file(false));
        assert!(tmp.path().join("inner2").contains_category_info_file());
    }
}
