#!/bin/bash

DEVCONTAINER_DIR="/workspaces/yomuyume/.devcontainer"

# git symlink
if [ ! -f /usr/local/bin/git ]; then
    if [ -f /usr/bin/git ]; then
        sudo ln -s /usr/bin/git /usr/local/bin/git
    fi
fi

# zsh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended

# just

# rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
echo 'export PATH="/usr/local/cargo/bin:$PATH"' >> ~/.zshrc

# volta, node. pnpm
curl https://get.volta.sh | bash
export VOLTA_HOME="$HOME/.volta"
export PATH="$VOLTA_HOME/bin:$PATH"
volta install node@lts pnpm
pnpm config set store-dir ~/.pnpm-store

# just
JUST_MD5=57f795ac704dca5b9e6fd4593ffafbb4
JUST_VERSION=1.40.0
if [ ! -f /usr/local/bin/just ]; then
    curl -L -o /tmp/just.tar.gz https://github.com/casey/just/releases/download/$JUST_VERSION/just-$JUST_VERSION-x86_64-unknown-linux-musl.tar.gz
    if [ "$(md5sum /tmp/just.tar.gz | awk '{print $1}')" = "$JUST_MD5" ]; then
        mkdir /tmp/just
        tar -xvf /tmp/just.tar.gz -C /tmp/just
        sudo mv /tmp/just/just /usr/local/bin
    else
        echo "just tarball does not match the expected md5"
    fi
    rm -rf /tmp/just*
fi

# fzf for just
FZF_VERSION=0.60.3
FZF_MD5=b9eccd3cb5ffeeaef1e85703bf0c9f75
if [ ! -f /usr/local/bin/fzf ]; then
    curl -L -o /tmp/fzf.tar.gz https://github.com/junegunn/fzf/releases/download/v$FZF_VERSION/fzf-$FZF_VERSION-linux_amd64.tar.gz
    if [ "$(md5sum /tmp/fzf.tar.gz | awk '{print $1}')" = "$FZF_MD5" ]; then
        sudo tar -xvf /tmp/fzf.tar.gz -C /usr/local/bin
    else
        echo "fzf-$FZF_VERSION-linux_amd64.tar.gz has been modified"
    fi
    rm -rf /tmp/fzf*
fi

echo 'alias j=just' >> ~/.zshrc

just install-wasmpack
just install-mold
just install-dav1d
just install-7z
