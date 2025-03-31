client_dir := justfile_dir() + "/src-frontend"
rust_dir := justfile_dir() + "/src-rust"

default:
    @just --choose

# lint the client codes
lint-c:
    cd {{client_dir}} && pnpm eslint --fix --cache .

# lint the rust codes
lint-s:
    cargo fmt && cargo clippy

# start the client dev server
dev-c:
    cd {{client_dir}} && pnpm nuxt dev

# start the server dev server
dev-s:
    cargo run -p yomuyume

# build the client
build-c:
    #!/usr/bin/env zsh
    cd {{client_dir}} && pnpm nuxt generate
    cp .nuxt/dist/client/manifest.webmanifest .output/public/manifest.webmanifest

bh_wsm_dist := client_dir / "components/image/blurhash-webp-wasm"
bh_wsm_src := rust_dir / "blurhash-webp-wasm"
jxl_wsm_dist := client_dir / "components/image/jxl-webp-wasm"
jxl_wsm_src := rust_dir / "jxl-webp-wasm"

_build_blurhash_wasm:
    #!/usr/bin/env zsh
    rm -rf {{bh_wsm_dist}}
    cd {{bh_wsm_src}}
    wasm-pack build --target web --out-dir {{bh_wsm_dist}}
    rm {{bh_wsm_dist}}/.gitignore

_build_jxl_wasm:
    #!/usr/bin/env zsh
    rm -rf {{jxl_wsm_dist}}
    cd {{jxl_wsm_src}}
    wasm-pack build --target web --out-dir {{jxl_wsm_dist}}
    rm {{jxl_wsm_dist}}/.gitignore

# build the wasm packages
build-w:
    just _build_blurhash_wasm
    just _build_jxl_wasm

# build the server
build-s +args="":
    cargo build -p yomuyume --release {{args}}

# upgrade client dependencies
upgrade-c:
    cd {{client_dir}} && pnpm upgrade

# upgrade rust dependencies
upgrade-s +args="":
    cargo update {{args}}