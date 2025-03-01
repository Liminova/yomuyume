FROM debian:12

RUN apt-get update && apt-get install -y \
    meson ninja-build nasm clang \
    pkg-config libssl-dev \
    curl git zsh sudo && apt-get clean

RUN groupadd -g 1000 ymym && \
    useradd -m -u 1000 -g ymym ymym && \
    echo "ymym ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/ymym && \
    mkdir -p /usr/local/cargo && chown -R ymym /usr/local/cargo && \
    mkdir -p /home/ymym/.pnpm-store && chown -R ymym /home/ymym/.pnpm-store

USER ymym

ENV CARGO_HOME=/usr/local/cargo