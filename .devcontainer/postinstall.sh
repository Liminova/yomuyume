#!/bin/bash

DEVCONTAINER_DIR="/workspaces/yomuyume/.devcontainer"
MOLD_VERSION="2.34.1"
DAV1D_VERSION="1.4.3"
DAV1D_MD5=c6fd9302a28d8c8e41e9a658a2be2031
MOLD_MD5=08d7304ea9f5e232a5c46a45f230b5db

echo "\n==============="
echo "= pnpm stuffs ="
echo "==============="
npm i -g pnpm
pnpm config set store-dir /home/vscode/.pnpm-store
rm -rf $DEVCONTAINER_DIR/../{node_modules,.nuxt}
pnpm install

echo "\n==================="
echo "= fix git symlink ="
echo "===================\n"
if [ ! -f /usr/local/bin/git ]; then
    if [ -f /usr/bin/git ]; then
        ln -s /usr/bin/git /usr/local/bin/git
    else
        echo "git is not installed"
    fi
else
    echo "git is already exist in /usr/local/bin"
fi

echo "\n=================="
echo "= download dav1d ="
echo "==================\n"
if [ ! -d "./dav1d-$DAV1D_VERSION" ]; then
    cd $DEVCONTAINER_DIR
    curl -L -o dav1d-$DAV1D_VERSION.tar.gz https://code.videolan.org/videolan/dav1d/-/archive/$DAV1D_VERSION/dav1d-$DAV1D_VERSION.tar.gz
    if [ "$(md5sum dav1d-$DAV1D_VERSION.tar.gz | awk '{print $1}')" = "$DAV1D_MD5" ]; then
        tar -xvf dav1d-$DAV1D_VERSION.tar.gz
        rm -f dav1d-$DAV1D_VERSION.tar.gz
    else
        echo "dav1d-$DAV1D_VERSION.tar.gz has been modified"
    fi
else
    echo "already downloaded dav1d-$DAV1D_VERSION"
fi

echo "\n==============="
echo "= build dav1d ="
echo "===============\n"
if [ ! -d $DEVCONTAINER_DIR/dav1d-$DAV1D_VERSION/build ]; then
    cd $DEVCONTAINER_DIR/dav1d-$DAV1D_VERSION
    mkdir build && cd build
    meson setup --default-library=static ..
    ninja
else
    echo "already built dav1d-$DAV1D_VERSION"
fi

echo "\n================="
echo "= symlink dav1d ="
echo "=================\n"
cd $DEVCONTAINER_DIR/dav1d-$DAV1D_VERSION/build && ninja install

echo "\n============================="
echo "= download and extract mold ="
echo "=============================\n"
if [ ! -d $DEVCONTAINER_DIR/mold-$MOLD_VERSION-x86_64-linux ]; then
    cd $DEVCONTAINER_DIR
    curl -L -o mold-$MOLD_VERSION-x86_64-linux.tar.gz https://github.com/rui314/mold/releases/download/v$MOLD_VERSION/mold-$MOLD_VERSION-x86_64-linux.tar.gz
    if [ "$(md5sum mold-$MOLD_VERSION-x86_64-linux.tar.gz | awk '{print $1}')" = "$MOLD_MD5" ]; then
        tar -xvf mold-$MOLD_VERSION-x86_64-linux.tar.gz
        rm -f mold-$MOLD_VERSION-x86_64-linux.tar.gz
    else
        echo "mold-$MOLD_VERSION-x86_64-linux.tar.gz has been modified"
    fi
else
    echo "already downloaded mold-$MOLD_VERSION-x86_64-linux"
fi

rm -f /root/cargo/config.toml
printf "[target.x86_64-unknown-linux-gnu]\nlinker = \"clang\"\nrustflags = [\"-C\", \"link-arg=-fuse-ld=/workspaces/yomuyume/.devcontainer/mold-$MOLD_VERSION-x86_64-linux/bin/mold\"]\n" > /root/cargo/config.toml
echo "\n======================================"
echo "= configure cargo to use mold linker ="
echo "======================================\n"

