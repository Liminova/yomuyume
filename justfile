default:
    @just --choose

# lint the client codes
lint-c:
    cd /ymym/client && \
        pnpm oxlint --import-plugin -D correctness -D perf && \
        pnpm prettier -l -w \"**/*.{js,ts,vue,json,css}\" && \
        cd -

# lint the rust codes
lint-s:
    cargo check && cargo fmt && cargo clippy

# start the client dev server
dev-c:
    cd /ymym/client && pnpm nuxt dev

# start the server dev server
dev-s:
    cargo run -p yomuyume

# build the client
build-c:
    #!/usr/bin/env zsh
    cd /ymym/client && pnpm nuxt generate
    if [ -f .nuxt/dist/client/manifest.webmanifest ]; then
        cp .nuxt/dist/client/manifest.webmanifest /ymym/client/.output/public/manifest.webmanifest
    fi

# build the server
build-s +args="":
    cargo build -p yomuyume --release {{ args }}

# upgrade client dependencies
upgrade-c:
    cd /ymym/client && pnpm upgrade && cd -

# upgrade rust dependencies
upgrade-s +args="":
    cargo update {{ args }}

build:
    #!/usr/bin/env zsh
    just build-w
    just gen-api-paths
    just build-c
    just build-s