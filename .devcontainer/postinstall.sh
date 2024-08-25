#!/bin/bash

DEVCONTAINER_DIR="/workspaces/yomuyume/.devcontainer"

sudo apt-get update && sudo apt-get install -y meson ninja-build nasm clang

# clone dav1d if not exists
if [ ! -d "./dav1d" ]; then
    cd $DEVCONTAINER_DIR
    git clone --depth 1 https://code.videolan.org/videolan/dav1d.git
fi

# build dav1d if not built
if [ ! -d $DEVCONTAINER_DIR/dav1d/build ]; then
    cd $DEVCONTAINER_DIR/dav1d
    mkdir build && cd build
    meson setup --default-library=static ..
    ninja
fi

# symlink dav1d
cd $DEVCONTAINER_DIR/dav1d/build && sudo ninja install

# download and extract mold if not exists
if [ ! -d $DEVCONTAINER_DIR/mold-2.33.0-x86_64-linux ]; then
    cd $DEVCONTAINER_DIR
    wget https://github.com/rui314/mold/releases/download/v2.33.0/mold-2.33.0-x86_64-linux.tar.gz
    tar -xvf mold-2.33.0-x86_64-linux.tar.gz
    rm -f mold-2.33.0-x86_64-linux.tar.gz
fi

# configure cargo to use mold linker
rm -f /usr/local/cargo/config.toml
printf "[target.x86_64-unknown-linux-gnu]\nlinker = \"clang\"\nrustflags = [\"-C\", \"link-arg=-fuse-ld=/workspaces/yomuyume/.devcontainer/mold-2.33.0-x86_64-linux/bin/mold\"]" > /usr/local/cargo/config.toml