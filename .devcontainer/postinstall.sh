#!/bin/bash

DEVCONTAINER_DIR="/workspaces/yomuyume/.devcontainer"
MOLD_VERSION="2.34.1"
DAV1D_VERSION="1.4.3"
SEVENZ_VERSION="2408"

DAV1D_MD5=c6fd9302a28d8c8e41e9a658a2be2031
MOLD_MD5=08d7304ea9f5e232a5c46a45f230b5db
SEVENZ_TAR_MD5=8908df4bec189cd1f314b54724911a36
SEVENZ_MD5=c7dce9920aac9217ae6ce2e35f18b985

echo "\n==============="
echo "= pnpm stuffs ="
echo "==============="
sudo npm i -g pnpm
pnpm config set store-dir /home/vscode/.pnpm-store
rm -rf $DEVCONTAINER_DIR/../{node_modules,.nuxt}
pnpm install

echo
echo "==================="
echo "= fix git symlink ="
echo "==================="
echo
if [ ! -f /usr/local/bin/git ]; then
    if [ -f /usr/bin/git ]; then
        sudo ln -s /usr/bin/git /usr/local/bin/git
    else
        echo "git is not installed"
    fi
else
    echo "git is already exist in /usr/local/bin"
fi

echo
echo "=================="
echo "= download dav1d ="
echo "=================="
echo
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

echo
echo "==============="
echo "= build dav1d ="
echo "==============="
echo
if [ ! -d $DEVCONTAINER_DIR/dav1d-$DAV1D_VERSION/build ]; then
    cd $DEVCONTAINER_DIR/dav1d-$DAV1D_VERSION
    mkdir build && cd build
    meson setup --default-library=static ..
    ninja
else
    echo "already built dav1d-$DAV1D_VERSION"
fi

echo
echo "================="
echo "= symlink dav1d ="
echo "================="
echo
cd $DEVCONTAINER_DIR/dav1d-$DAV1D_VERSION/build && sudo ninja install

echo
echo "============================="
echo "= download and extract mold ="
echo "============================="
echo
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

echo
echo "======================================"
echo "= configure cargo to use mold linker ="
echo "======================================"
echo
rm -f /home/node/.cargo/config.toml && mkdir -p /home/node/.cargo && touch /home/node/.cargo/config.toml
printf "[target.x86_64-unknown-linux-gnu]\nlinker = \"clang\"\nrustflags = [\"-C\", \"link-arg=-fuse-ld=/workspaces/yomuyume/.devcontainer/mold-$MOLD_VERSION-x86_64-linux/bin/mold\"]\n" > /home/node/.cargo/config.toml
echo "cargo config created"

# 7zip
echo
echo "================"
echo "= download 7zz ="
echo "================"
echo
if [ ! -f $DEVCONTAINER_DIR/7zz ] || [ "$(md5sum $DEVCONTAINER_DIR/7zz | awk '{print $1}')" != "$SEVENZ_MD5" ]; then
    rm -f /tmp/7z.tar.xz $DEVCONTAINER_DIR/7zz
    curl -L -o /tmp/7z.tar.xz https://www.7-zip.org/a/7z$SEVENZ_VERSION-linux-x64.tar.xz
    if [ -f /tmp/7z.tar.xz ]; then
        if [ "$(md5sum /tmp/7z.tar.xz | awk '{print $1}')" = "$SEVENZ_TAR_MD5" ]; then
            mkdir -p /tmp/7z && tar -xvf /tmp/7z.tar.xz -C /tmp/7z
            mv /tmp/7z/7zz $DEVCONTAINER_DIR/7zz
            rm -rf /tmp/7z2408-linux-x64
        else
            echo "7z2408-linux-x64.tar.xz has been modified"
        fi
    else
        echo "7z2408-linux-x64.tar.xz not found"
    fi
else
    echo "7zz already exist in $DEVCONTAINER_DIR"
fi

echo