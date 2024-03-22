# yomuyume

<img src="assets/yomuyume.png" alt="yomuyume logo" width="128" height="128">

this is the main monorepo for Yomuyume, a media server for manga and comics.

## repository info

this monorepo contains the following codebases:

-   `src/` - the Nuxt.js frontend component.

the reference implementation of Yomuyume's frontend.

-   `src-rust/` - the Rust/Axum-based backend component + Rust->Wasm bridge

this contains all the API calls, along with interfacing with the SQLite database.

## developement

- the server and client can be developed independently, check `package.json` for the available scripts.
- `pnpm build-bridge` must be run at least once before `pnpm build-client`.

## deployment

follow [deploy](./deploy/README.md) guide

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
