FROM node:lts-bookworm-slim

RUN apt-get update && apt-get install -y \
    # dav1d
    meson ninja-build nasm \
    # mold linker
    clang \
    # openssl crate somewhere in the dependency tree
    pkg-config libssl-dev
    # misc
    curl git zsh && apt-get clean

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y