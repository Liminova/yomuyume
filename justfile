default:
    @just --choose

# lint the client codes
lint-c:
    cd /ymym/client && \
        pnpm oxlint --fix \
            -D correctness \
            -D suspicious \
            -D pedantic \
            -D perf \
            -A group-export \
            -A no-null \
            -A filename-case \
            -A consistent-type-specifier-style \
            -A max-lines-per-function \
            -A sort-imports \
            -A prefer-global-this \
            -A func-style \
            -A sort-keys \
            -A prefer-add-event-listener \
            -A require-post-message-target-origin \
            -A new-cap \
            -A no-magic-numbers \
            -A id-length && \
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