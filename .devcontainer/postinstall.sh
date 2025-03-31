#!/bin/bash

DEVCONTAINER_DIR="/workspaces/yomuyume/.devcontainer"
MOLD_VERSION=2.36.0
DAV1D_VERSION="1.5.0"
SEVENZ_VERSION="2408"
FZF_VERSION=0.60.3

DAV1D_MD5=dda9e056e8dc95471a1126308c18868d
MOLD_MD5=0cbdd068a70ef28cad32c4005fd9f1df
SEVENZ_TAR_MD5=8908df4bec189cd1f314b54724911a36
SEVENZ_MD5=c7dce9920aac9217ae6ce2e35f18b985
FZF_MD5=b9eccd3cb5ffeeaef1e85703bf0c9f75

cd "/workspaces/yomuyume"

# rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# zsh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended

# git symlink
if [ ! -f /usr/local/bin/git ]; then
    if [ -f /usr/bin/git ]; then
        sudo ln -s /usr/bin/git /usr/local/bin/git
    fi
fi

# download dav1d
if [ ! -d "./dav1d-$DAV1D_VERSION" ]; then
    cd $DEVCONTAINER_DIR
    curl -L -o dav1d-$DAV1D_VERSION.tar.gz https://code.videolan.org/videolan/dav1d/-/archive/$DAV1D_VERSION/dav1d-$DAV1D_VERSION.tar.gz
    if [ "$(md5sum dav1d-$DAV1D_VERSION.tar.gz | awk '{print $1}')" = "$DAV1D_MD5" ]; then
        tar -xvf dav1d-$DAV1D_VERSION.tar.gz
        rm -f dav1d-$DAV1D_VERSION.tar.gz
    else
        echo "dav1d-$DAV1D_VERSION.tar.gz has been modified"
    fi
fi

# build dav1d
if [ ! -d $DEVCONTAINER_DIR/dav1d-$DAV1D_VERSION/build ]; then
    cd $DEVCONTAINER_DIR/dav1d-$DAV1D_VERSION
    mkdir build && cd build
    meson setup --default-library=static ..
    ninja
fi

# install dav1d
cd $DEVCONTAINER_DIR/dav1d-$DAV1D_VERSION/build && sudo ninja install

# download mold
if [ ! -d /usr/local/cargo/mold-$MOLD_VERSION-x86_64-linux ]; then
    cd /usr/local/cargo
    curl -L -o mold-$MOLD_VERSION-x86_64-linux.tar.gz https://github.com/rui314/mold/releases/download/v$MOLD_VERSION/mold-$MOLD_VERSION-x86_64-linux.tar.gz
    if [ "$(md5sum mold-$MOLD_VERSION-x86_64-linux.tar.gz | awk '{print $1}')" = "$MOLD_MD5" ]; then
        tar -xvf mold-$MOLD_VERSION-x86_64-linux.tar.gz
        rm -f mold-$MOLD_VERSION-x86_64-linux.tar.gz
    else
        echo "mold-$MOLD_VERSION-x86_64-linux.tar.gz has been modified"
    fi
fi

# configure cargo to use mold
rm -f /usr/local/cargo/config.toml
printf "[target.x86_64-unknown-linux-gnu]\nlinker = \"clang\"\nrustflags = [\"-C\", \"link-arg=-fuse-ld=/usr/local/cargo/mold-$MOLD_VERSION-x86_64-linux/bin/mold\"]" > /usr/local/cargo/config.toml

# 7zip
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
fi

# use binstall so we don't have to compile
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
echo 'export PATH="/usr/local/cargo/bin:$PATH"' >> ~/.zshrc

cargo binstall -y wasm-pack just

# install volta
curl https://get.volta.sh | bash
export VOLTA_HOME="$HOME/.volta"
export PATH="$VOLTA_HOME/bin:$PATH"

# node, pnpm
volta install node@lts pnpm
pnpm config set store-dir ~/.pnpm-store
pnpm i

# fzf for just
if [ ! -f /usr/local/bin/fzf ]; then
    curl -L -o /tmp/fzf.tar.gz https://github.com/junegunn/fzf/releases/download/v$FZF_VERSION/fzf-$FZF_VERSION-linux_amd64.tar.gz
    if [ "$(md5sum /tmp/fzf.tar.gz | awk '{print $1}')" = "$FZF_MD5" ]; then
        sudo tar -xvf /tmp/fzf.tar.gz -C /usr/local/bin
    else
        echo "fzf-$FZF_VERSION-linux_amd64.tar.gz has been modified"
    fi
    rm -f /tmp/fzf.tar.gz
fi