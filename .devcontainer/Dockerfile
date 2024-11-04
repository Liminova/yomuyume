FROM node:lts-bookworm-slim

RUN apt-get update && apt-get install -y \
    # dav1d
    meson ninja-build nasm \
    # mold linker
    clang \
    # openssl crate somewhere in the dependency tree
    pkg-config libssl-dev \
    # misc
    curl git zsh sudo && apt-get clean

RUN usermod -l ymym node && groupmod -n ymym node && \
    echo "ymym ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers && \
    usermod --uid 1000 ymym && groupmod --gid 1000 ymym && \
    mkdir -p /usr/local/cargo /home/vscode/.pnpm-store && \
    chown -R ymym /usr/local/cargo /home/vscode/.pnpm-store

USER ymym

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal -c clippy rustfmt

ENV CARGO_HOME=/usr/local/cargo