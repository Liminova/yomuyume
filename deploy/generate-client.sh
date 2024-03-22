#!/bin/sh

if [ "$1" = "--prelude" ]; then
    rm -rf yomuyume
    apk add --no-cache git
    git clone --depth 1 https://github.com/Liminova/yomuyume.git

elif [ "$1" = "--build-bridge" ]; then
    cd /app/yomuyume
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    sh ./src-rust/build-bridge.sh

elif [ "$1" = "--generate" ]; then
    cd /app/yomuyume
    npm install -g pnpm
    pnpm i && pnpm nuxt generate
    mv .output/public ../yomuyume-client && cd ..
    rm -rf /app/yomuyume

else
    echo "=== STAGE 1: PRELUDE ==="
    docker run --rm -v "$(pwd)":/app -w /app node:21-alpine sh generate-client.sh --prelude

    echo "=== STAGE 2: BUILD WASM BRIDGE ==="
    docker run --rm -v "$(pwd)":/app -w /app rust:bookworm sh generate-client.sh --build-bridge

    echo "=== STAGE 3: GENERATE CLIENT ==="
    docker run --rm -v "$(pwd)":/app -w /app node:21-alpine sh generate-client.sh --generate
fi