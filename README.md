# Yomuyume

<div align="center">
  <img src="src/public/favicon/android-chrome-192x192.png" alt="Yomuyume logo" width="192" height="192">

  Self-hosted media server for manga and comics, written in Rust.
</div>

## highlights

- ⚡ written in Rust for blazingly fast performance,

- 🛡️ no complex intermediate data, your library is the **source of truth**,
- 🧠 automatically detect one shots, series, and **categories**,
- ✅ compatible with your existing library directory structure,
  > including but not limited to [`#recycle`](https://komga.org/docs/guides/libraries#directory-exclusions) and [`_oneshots`](https://komga.org/docs/guides/oneshots/) hacks in directory paths
- 🗃️ support all the archive formats thanks to `7zip`,
- 📄 manage metadata using (a modified version of) the latest version of [The Anansi Project's `ComicInfo.xml` schema](https://anansi-project.github.io/docs/comicinfo/intro),
- 🌫️ [blurha.sh](https://blurha.sh) as placeholder for loading images,
- ⭕ respect `.nomedia` files,

and many more.

## deploy
see [docs/deployment.md](docs/deployment.md)

## managing your library
see [docs/managing-library.md](docs/managing-library.md)

## develop
see [docs/development.md](docs/development.md)

## license

licensed under either of

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
-   MIT license
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## contribution

unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
