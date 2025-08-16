# Managing Your Library

We won't tell you how you should manage your library—it's yours after all! However, this application doesn't possess _human-level intelligence_ to accommodate every possible use case. Therefore, this guide will explain how Yomuyume **views** your library, allowing you to reorganize it (hopefully with minimal adjustments) so that it understands.

## I. Supported Formats

-   Archive files: any format that 7zip supports. However, for performance reasons, we maintain a static list of file extensions considered as archive files in the relase binary of the application. These includes

    -   `7z`
    -   `cbz`
    -   `rar`
    -   `cbr`
    -   `zip`

    Create a new issue or pull request if you'd like us to add more.

-   Image formats: you can use any image format supported by your browser, as Yomuyume directly streams files from your storage to the client, without any intermediate processing. For performance reasons, we also maintain a list of file extensions recognized as image files:

    -   `avif`
    -   `bmp`
    -   `gif`
    -   `jpg`/`jpeg`
    -   `png`
    -   `tif`/`tiff`
    -   `webp`
    -   `jxl` (w/ a caveat in the [Cover Image](#iv-cover-image) section)

    The application only decodes the image during the search/validation of cover images, which will be addressed in the [Cover Image](#iv-cover-image) section.

## II. Directory Structure

<!-- https://tree.nathanfriend.com/?s=(%27options!(%27fancy!true~fullPath7~trailingSlash7~rootDot7)~E(%27E%27library%2F5Category%20AF4AG3cover.png*3page01.avif*39*4BF34B%20-H183954CFcover8018954D%2F61.zip62G9%27)~version!%271%27)*533%20%204Title%205%5Cn36*ChapterH7!false8.jpg*9...Esource!F%2F*G.cbz*H%200%01HGFE9876543* -->

```
library/
├── Category A/
│   ├── Title A.cbz
│   │   ├── cover.png
│   │   ├── page01.avif
│   │   └── ...
│   └── Title B/
│       ├── Title B - 01.jpg
│       └── ...
├── Title C/
│   ├── cover.jpg
│   ├── 01.jpg
│   └── ...
└── Title D/
    ├── Chapter 01.zip
    ├── chapter_02.cbz
    └── ...
```

Consider this minimalistic example of a library organized by titles and categories, featuring a category named `Category A` which includes two one-shots: `Title A` as an archive file and `Title B` as a directory.

Additionally, `Title C` and `Title D` are located in the root directory, categorizing them as non-categorized titles. `Title A`, `Title B`, and `Title C` are one-shots, whereas `Title D` is a series.

In this system, a directory can function as _a one-shot title_, _a series_, or _a category_, based on a decision tree detailed in [dir-entry-guesser.md](diagrams/dir-entry-guesser.md). The summary of the rules is as follows:

-   A directory is considered a series if all sub-directories and archive files inside follow a naming pattern of `<optional-basename><chapter_number>`. Extensions are disregarded, and basenames are normalized by lowercasing and removing non-alphanumeric characters.

    extensions are ignored so this is a valid series:

    -   `Title A/chapter_001`
    -   `Title A/chapter_002.cbz`
    -   `Title A/chapter_003.7z`

    basenames are lowercased and filter out all non-alphanumeric characters so this is also valid:

    -   `Title B/Story-001`
    -   `Title B/story 002.cbr`
    -   `Title B/story_003.zip`

-   A directory is recognized as a one-shot title if it contains more than one image file.

    -   `Title C/cover.jpg`
    -   `Title C/01.jpg`
    -   `Title C/02.jpg`

-   If neither condition is met, it is considered a category.

The rule requiring "more than one image file" ensures that a category has a cover image. If a directory contains more than one image but is intended to be treated as a category, add an empty `CategoryInfo.xml` file to the directory.

## III. Metadata

### `ComicInfo.xml`

For managing metadata management, Yomuyume uses a [slightly extended version](https://github.com/Delnegend/Delnegend/blob/main/comicinfo-2.1-extended.xsd) of [`ComicInfo.xml` v2.1](https://anansi-project.github.io/docs/comicinfo/schemas/v2.1) schema for titles, including one-shots, series, and chapters.

There're only 4 attributes added to the `ComicPageInfo` type from the `ComicInfo.xml` schema in the `ComicPageInfo` type:

```xml
...
    <xs:complexType name="ComicPageInfo">
        ...
        <xs:attribute default="" name="ImagePath" type="xs:string"/>
        <xs:attribute default="" name="Description" type="xs:string"/>
        <xs:attribute default="" name="Blurhash" type="xs:string"/>
        <xs:attribute default="" name="ModifiedDateAtEncode" type="xs:string"/>
    </xs:complexType>
...
```

-   `ImagePath`: We could've use the `Image` attribute but it's a number field, we presumed it's linking to image files by their order in the title but this doesn't guarantee the order when, let's say you remove an image in the middle of the title.

    Use paths **relative to the `ComicInfo.xml` file only**, omitting the leading `./` or `.\`, and avoid using `..` or `.` to navigate out of the current context. For instance, consider the following library structure:

      <!-- https://tree.nathanfriend.com/?s=(%27options!(%27fancy!true~fullPath4~trailingSlash4~rootDot4)~5(%275%27Library%2F3A6cover8alt6-01823B6Chapter%201.zip*-2*-01.png*2%27)~version!%271%27)*7--%20%202ComicInfo.xml37Title%204!false5source!6%2F*7%5Cn-8.jpg*%018765432-* -->

    ```
    Library/
    ├── Title A/
    │   ├── cover.jpg
    │   ├── alt/
    │   │   └── 01.jpg
    │   └── ComicInfo.xml
    └── Title B/
        ├── Chapter 1.zip
        │   ├── ComicInfo.xml
        │   └── 01.png
        └── ComicInfo.xml
    ```

    -   For `Title A`, the image paths are
        ```xml
        <ComicInfo>
            ...
            <Pages>
                ...
                <Page ImagePath="cover.jpg" ... />
                <Page ImagePath="alt/01.jpg" ... />
            </Pages>
        </ComicInfo>
        ```
    -   For Chapter 1 of Title B: `<Page ImagePath="01.png" ... />`
    -   For Title B: `<Page ImagePath="Chapter 1.zip" ... />`

-   `Description`: A a free-form text field for describe the page, add notes, or anything else you want to add.

-   [`Blurhash`](https://blurha.sh/): _"A compact representation of a placeholder for an image,"_ we only use this for the cover images so there's no need to fill it in for all the pages, and it's **auto-generated** by Yomuyume.

-   `ModifiedDateAtEncode`: the modified date of the page when it was encoded into blurhash, this is used to determine whether the blurhash is outdated or not, also **auto-filled** by Yomuyume.

### `CategoryInfo.xml`

The schema can be found [here](https://github.com/Delnegend/Delnegend/blob/main/categoryinfo-1.0.xsd), but TL;DR:

```xml
<CategoryInfo>
    <Name>Adventure</Name>
    <Description>Lorem Ipsum</Description>
    <Cover>
        <Path>adventure.jpg</Path>
        <Blurhash>LEHV6nWB2yk8pyo0adR*.7kCMdnj</Blurhash>
        <Width>300</Width>
        <Height>200</Height>
        <ModifiedDateAtEncode>2024-01-01T00:00:00Z</ModifiedDateAtEncode>
    </Cover>
</CategoryInfo>
```

You just need to care about the `Name`, `Description`, and `Path` in the `<Cover>` section, everything else leave to Yomuyume.

This `Path` has the same rules as the `ImagePath` in `ComicInfo.xml`, always relative to the `CategoryInfo.xml` file without the leading `./` or `.\`..

> Tips: Open the `xml`s using [vscode](https://code.visualstudio.com/) with the [XML](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-xml) extension to have type hinting, or you can use the frontend client.

## IV. Cover Image

Path to cover images are specified in the `ComicInfo.xml` and `CategoryInfo.xml` files.

-   With `ComicInfo.xml`, Yomuyume will try to find a `<Page>` element in the `<Pages>` section with these 2 attributes **FROM THE BOTTOM UP**:

    -   `ComicPageType` is `FrontCover`
    -   `ImagePath` points to a valid image file

    and check if these properties are also met:

    -   `Width` > 0 and `Height` > 0
    -   `Blurhash` is decode-able
    -   `ModifiedDateAtEncode` is the same as the file's modified date of the specified file in `ImagePath`

    then it will use it, or else it will exhaustively check **every single image** in the title until it finds one that can be decoded to be encoded into blurhash.

-   With `CategoryInfo.xml`, Yomuyume will only try to use the configured cover image before falling back to a generic one.

> Caveat of JPEG XL: we haven't added a decoder for it yet, so if you have a cover image in JPEG XL, fill in the `Blurhash`, `LastModifiedAt`, `Width` and `Height` attributes manually. Transcode the cover to JPEG first and go to [Blurha.sh](https://blurha.sh/) to generate the `Blurhash` string. You can keep the `.jxl` cover file in your title, but make sure its last modified date is the same as the one in `LastModifiedAt`.

## V. Source of Truth

Everything that can be stored will be located within `ComicInfo.xml` and `CategoryInfo.xml`. **Your library** serves as the definitive source of truth, **not the application's database**.

## VI. Special Features

### 1. Titles's Extras

Artists often include artwork or mini-comics in their titles. If you prefer not to have them appear in the main storyline, if you're unsure how to organize them, or if there are different language versions of the same title that you don't want to be displayed as separate titles, you can organize them into subdirectories of the title like so:

<!-- https://tree.nathanfriend.com/?s=(%27options!(%27fancy!true~fullPath4~trailingSlash4~rootDot4)~6(%276%27Library%2F7Title%20A8cover5*...*Extras3*French3%27)~version!%271%27)*7--%20%2038-015*-0254!false5.png6source!7%5Cn-8%2F*%01876543-* -->

```
Library/
└── Title A/
    ├── cover.png
    ├── ...
    ├── Extras/
    │   ├── 01.png
    │   └── 02.png
    └── French/
        ├── 01.png
        └── 02.png
```

Note that only the last element of the path is used as the extra's title, so `French/` and `Extras/French/` would be considered the same thing. Instead, use `Extras (French)/`.

For convenience, the metadata for these extras' pages is managed in the same `ComicInfo.xml` file at the root of the title, for example:

```xml
<Pages>
    <Page ImagePath="Extras/01.png" ... />
    <Page ImagePath="Extras/02.png" ... />
    <Page ImagePath="French/01.png" ... />
    <Page ImagePath="French/02.png" ... />
    ...
</Pages>
```

### 2. The `.nomedia`

Creating an empty `.nomedia` file within a directory instructs Yomuyume to omit the entire directory tree from its scanning process. This method is also applicable to archives and directories contained within them just like with directories. By **default**, this feature is **disabled**.

### 3. Markdown

Any text field in both `ComicInfo.xml` and `CategoryInfo.xml` can be formatted using Markdown. Just remember to escape these characters that will have conflicts with the XML syntax:

| Character | Escaped  |
| --------- | -------- |
| `"`       | `&quot;` |
| `'`       | `&apos;` |
| `<`       | `&lt;`   |
| `>`       | `&gt;`   |
| `&`       | `&amp;`  |

## VII. Komga compatibility features

Komga allows you to

-   [Ignore a directory from being scanned by adding `#recycle` into its path](https://komga.org/docs/guides/oneshots/).
-   [Force all titles inside a directory to be one-shots by adding `_oneshot` into its path](https://komga.org/docs/guides/libraries#directory-exclusions).

TL;DR: to ignore a directory from being scanned in Komga,

-   prefix `#recycle` to the directory path to ignore only the directory itself

    `/my-library/#recycle foo/bar/baz` -> only `#recycle foo` is ignored, `bar` and `baz` are scanned

-   add `#recycle` anywhere in the path to ignore it and its subdirectories

    `/my-library/foo #recycle foo2/bar/baz` -> all `foo #recycle foo2`, `bar` and `baz` are ignored

samething with the `_oneshot` flag.

`_oneshot` and `#recycle` are only applied when deciding whether a directory is a series or a one-shot title, if the application detects a `DirEntry` is a category, then it's a category. Both features are **disabled by default** in the settings page in the frontend.

The Mylar's `series.json` will be supported in the far future.
