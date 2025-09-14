#!/bin/bash

# zsh
sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended

# rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
echo 'export PATH="/usr/local/cargo/bin:$PATH"' >> ~/.zshrc

# js things
curl https://get.volta.sh | bash
export VOLTA_HOME="$HOME/.volta" && export PATH="$VOLTA_HOME/bin:$PATH"
echo 'export VOLTA_HOME="$HOME/.volta"' >> ~/.zshrc
echo 'export PATH="$VOLTA_HOME/bin:$PATH"' >> ~/.zshrc
volta install node@lts pnpm
pnpm config set store-dir ~/.pnpm-store
cd /ymym/client && pnpm i && cd -

# just
ver="1.40.0"
curl -L -o /tmp/just.tar.gz https://github.com/casey/just/releases/download/$ver/just-$ver-x86_64-unknown-linux-musl.tar.gz
checksum=$(openssl dgst -sha3-512 /tmp/just.tar.gz | awk '{print $2}')
expected="27f317b6ca704395dbad34c078f57d140d6d3e1f147a29b5c7563489885525a203bb289b02ec05e52282d25dfe446a60e11554a58addc682ad17f94b6b100cb9"
if [ ! "$checksum" = "$expected" ]; then
    echo "just tarball checksum failed\nexpected: $expected\ngot: $checksum"
else
    sudo tar -xf /tmp/just.tar.gz -C /usr/local/bin just
    just --completions zsh > ~/.just.zsh
    echo '[[ -f ~/.just.zsh ]] && source ~/.just.zsh' >> ~/.zshrc
    if [[ -f /usr/local/bin/just ]]; then
        echo "just installed successfully"
    else
        echo "just installation failed"
    fi
fi
rm -f /tmp/just.tar.gz
echo 'alias j=just' >> ~/.zshrc

# fzf
ver="0.61.3"
curl -L -o /tmp/fzf.tar.gz https://github.com/junegunn/fzf/releases/download/v$ver/fzf-$ver-linux_amd64.tar.gz
checksum=$(openssl dgst -sha3-512 /tmp/fzf.tar.gz | awk '{print $2}')
expected="1710205b6f924c78ebfc6b43e1697e4cf4ba168d7970196f23effb4f125e956a76a07ae8a26dfcd1a4a5b26435b2670bb840b7d1c4ea92befef09789d17068b0"
if [ ! "$checksum" = "$expected" ]; then
    echo "fzf tarball checksum failed\nexpected: $expected\ngot: $checksum"
else
    sudo tar -xf /tmp/fzf.tar.gz -C /usr/local/bin fzf
    if [[ -f /usr/local/bin/fzf ]]; then
        echo "fzf installed successfully"
    else
        echo "fzf installation failed"
    fi
fi
rm -f /tmp/fzf.tar.gz

# mold
ver="2.37.1"
curl -L -o /tmp/mold.tar.gz https://github.com/rui314/mold/releases/download/v$ver/mold-$ver-x86_64-linux.tar.gz
checksum=$(openssl dgst -sha3-512 /tmp/mold.tar.gz | awk '{print $2}' | tr -d '\n')
expected="bab38238011b77430fae4509d62cb7f175845afe6a81e83a10be16570d149fad440d179f93e0d464405749af58ea3581a17e94f7650d88043b359235db1a5545"
if [ ! "$checksum" = "$expected" ]; then
    echo "mold tarball checksum failed\nexpected: $expected\ngot: $checksum"
else
    sudo rm -rf /usr/local/cargo/mold-$ver-x86_64-linux
    sudo tar -xf /tmp/mold.tar.gz -C /usr/local/cargo
    if [ -f /usr/local/cargo/mold-$ver-x86_64-linux/bin/mold ]; then
        echo "mold installed successfully"
    else
        echo "mold installation failed"
    fi
fi
rm -f /tmp/mold.tar.gz

# configure cargo to use mold
rm -f /usr/local/cargo/config.toml
printf "[target.x86_64-unknown-linux-gnu]\nlinker = \"clang\"\nrustflags = [\"-C\", \"link-arg=-fuse-ld=/usr/local/cargo/mold-$ver-x86_64-linux/bin/mold\"]" > /usr/local/cargo/config.toml
echo "cargo config: \n$(cat /usr/local/cargo/config.toml)"

# get & extract dav1d
ver="1.5.0"
if [ -d "/usr/local/lib/dav1d/$ver" ]; then
    echo "dav1d $ver already installed, skipping"
else
    curl -L -o /tmp/dav1d.tar.gz https://code.videolan.org/videolan/dav1d/-/archive/$ver/dav1d-$ver.tar.gz
    checksum=$(openssl dgst -sha3-512 /tmp/dav1d.tar.gz | awk '{print $2}' | tr -d '\n')
    expected="eab0a27f56576233b4a23f227df77d761ad566333363ac0f9b8babe83990f672a49fa4d7dc79f4596e78bf2f91465081e2ce289dc6b0f5a2e9d120e0c1504291"

    if [ "$checksum" != "$expected" ]; then
        echo "dav1d tarball checksum failed"
        exit 1
    else
        sudo mkdir -p /usr/local/lib/dav1d/$ver
        sudo tar -xf /tmp/dav1d.tar.gz -C /usr/local/lib/dav1d/$ver --strip-components=1
    fi
    rm -f /tmp/dav1d.tar.gz
fi

# build dav1d
if [ ! -d /usr/local/lib/dav1d/$ver/build ]; then
    cd /usr/local/lib/dav1d/$ver
    sudo mkdir build && cd build
    sudo meson setup --default-library=static ..
    sudo ninja
fi
cd /usr/local/lib/dav1d/$ver/build && sudo ninja install

if ! command -v 7zz >/dev/null; then
    # 7zip
    ver="2501"
    curl -L -o /tmp/7z.tar.xz https://www.7-zip.org/a/7z$ver-linux-x64.tar.xz
    checksum=$(openssl dgst -sha3-512 /tmp/7z.tar.xz | awk '{print $2}')
    expected="20ab025c487de16840ca4b4a5783ae62bda66ddbb4488ea4a91c0e521cee9dc90513a93bc52694c1c23e7208c6f03608e1a418c6612ebed19437114fa3933cee"
    if [ "$checksum" != "$expected" ]; then
        echo "7z tarball checksum failed\nexpected: $expected\ngot: $checksum"
    else
        sudo tar -xf /tmp/7z.tar.xz -C /usr/local/bin/ 7zz
        [[ -f /usr/local/bin/7zz ]] || { echo "7zz not found"; exit 1; }
    fi
    rm -f /tmp/7z.tar.xz
else
    echo "7zz already installed, skipping"
fi

# sqlx-cli
/usr/local/cargo/bin/cargo install sqlx-cli
/usr/local/cargo/bin/sqlx completions zsh > ~/.sqlx.zsh
echo '[[ -f ~/.sqlx.zsh ]] && source ~/.sqlx.zsh' >> ~/.zshrc