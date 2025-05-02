client_dir := "src-frontend"
client_dist_dir := client_dir + "/.output/public"

rust_dir := justfile_dir() + "/src-rust"
build_rs_file := rust_dir + "/server/build.rs"

client_dir_abs := justfile_dir() + "/" + client_dir
client_dist_dir_abs := justfile_dir() + "/" + client_dist_dir

default:
    @just --choose

install-7z:
    #!/usr/bin/env zsh
    ver="2408"
    curl -L -o /tmp/7z.tar.xz https://www.7-zip.org/a/7z$ver-linux-x64.tar.xz
    checksum=$(openssl dgst -sha3-512 /tmp/7z.tar.xz | awk '{print $2}')
    expected="5c0e23cc2575219d5a2962cdf66adfc117454291f80e6786c55267ff1c076e645fb62dbe1d7ab60de9a9de473af602d1e4dbf3cdaaf210379fb84ba1e9eb4996"

    if [ ! "$checksum" = "$expected" ]; then
        rm -f /tmp/7z.tar.xz
        echo "7z tarball checksum failed\nexpected: $expected\ngot: $checksum"
    else
        rm -f src-rust/server/utils/7zz
        tar -xf /tmp/7z.tar.xz -C src-rust/server/utils 7zz
        [[ -f src-rust/server/utils/7zz ]] || { echo "7z not found"; exit 1; }
    fi
    rm -f /tmp/7z.tar.xz

install-dav1d:
    #!/usr/bin/env bash
    ver="1.5.0"
    curl -L -o /tmp/dav1d.tar.gz https://code.videolan.org/videolan/dav1d/-/archive/$ver/dav1d-$ver.tar.gz
    checksum=$(openssl dgst -sha3-512 /tmp/dav1d.tar.gz | awk '{print $2}' | tr -d '\n')
    expected="eab0a27f56576233b4a23f227df77d761ad566333363ac0f9b8babe83990f672a49fa4d7dc79f4596e78bf2f91465081e2ce289dc6b0f5a2e9d120e0c1504291"

    if [ ! "$checksum" = "$expected" ]; then
        rm -f /tmp/dav1d.tar.gz
        echo "dav1d tarball checksum failed"
        exit 1
    else
        sudo rm -rf /usr/local/lib/dav1d-$ver
        sudo tar -xf /tmp/dav1d.tar.gz -C /usr/local/lib
        rm -f /tmp/dav1d.tar.gz
    fi

    # build dav1d
    if [ ! -d /usr/local/lib/dav1d-$ver/build ]; then
        cd /usr/local/lib/dav1d-$ver
        sudo mkdir build && cd build
        sudo meson setup --default-library=static ..
        sudo ninja
    fi

    # install dav1d
    cd /usr/local/lib/dav1d-$ver/build && sudo ninja install

install-mold:
    #!/usr/bin/env bash
    ver="2.37.1"
    curl -L -o /tmp/mold.tar.gz https://github.com/rui314/mold/releases/download/v$ver/mold-$ver-x86_64-linux.tar.gz
    checksum=$(openssl dgst -sha3-512 /tmp/mold.tar.gz | awk '{print $2}' | tr -d '\n')
    expected="bab38238011b77430fae4509d62cb7f175845afe6a81e83a10be16570d149fad440d179f93e0d464405749af58ea3581a17e94f7650d88043b359235db1a5545"

    if [ ! "$checksum" = "$expected" ]; then
        rm -f /tmp/mold.tar.gz
        echo "mold tarball checksum failed\nexpected: $expected\ngot: $checksum"
        exit 1
    else
        sudo rm -rf /usr/local/cargo/mold-$ver-x86_64-linux
        sudo tar -xf /tmp/mold.tar.gz -C /usr/local/cargo
        rm -f /tmp/mold.tar.gz
    fi

    # configure cargo to use mold
    rm -f /usr/local/cargo/config.toml
    printf "[target.x86_64-unknown-linux-gnu]\nlinker = \"clang\"\nrustflags = [\"-C\", \"link-arg=-fuse-ld=/usr/local/cargo/mold-$ver-x86_64-linux/bin/mold\"]" > /usr/local/cargo/config.toml
    echo "cargo config: \n$(cat /usr/local/cargo/config.toml)"

wasmpack_ver := "0.13.1"
wasmpack_md5 := "60e58da7aac6ea343fa2500dd92ae061"

install-wasmpack:
    #!/usr/bin/env bash
    ver="0.13.1"
    curl -L -o /tmp/wasm-pack.tar.gz https://github.com/rustwasm/wasm-pack/releases/download/v$ver/wasm-pack-v$ver-x86_64-unknown-linux-musl.tar.gz
    checksum=$(openssl dgst -sha3-512 /tmp/wasm-pack.tar.gz | awk '{print $2}' | tr -d '\n')
    expected="e229a8e8c626fbb65bc7b4bb623b4557d80150ae33bacefa81dd2086a5f13c411642f3e5a18614cc81d40e0d759ba275643f07fea4ce6f59015402e8d9669c32"

    if [ ! "$checksum" = "$expected" ]; then
        rm -f /tmp/wasm-pack.tar.gz
        echo "wasm-pack tarball checksum failed\nexpected: $expected\ngot: $checksum"
        exit 1
    else
        sudo rm -rf /usr/local/bin/wasm-pack
        sudo tar -xf /tmp/wasm-pack.tar.gz -C /usr/local/bin --strip-components=1 wasm-pack-v$ver-x86_64-unknown-linux-musl/wasm-pack
        rm -f /tmp/wasm-pack.tar.gz
        [[ -f /usr/local/bin/wasm-pack ]] || { echo "wasm-pack not found"; exit 1; }
    fi

install-frontend-deps:
    cd {{ client_dir_abs }} && pnpm i

_ensure_client_dist_dir_for_dev:
    #!/usr/bin/env python3
    import os, shutil

    os.makedirs("{{ client_dist_dir_abs }}", exist_ok=True)
    if not os.path.exists("{{ client_dist_dir_abs }}/index.html"):
        with open("{{ client_dist_dir_abs }}/index.html", "w") as f:
            f.write("<!DOCTYPE html><html><head><title>Hello World!</title></head><body>Hello World!</body></html>")

# lint the client codes
lint-c: gen-api-paths
    cd {{ client_dir_abs }} && pnpm eslint --fix --cache .

# lint the rust codes
lint-s: _ensure_client_dist_dir_for_dev
    cargo check && cargo fmt && cargo clippy

# start the client dev server
dev-c: gen-api-paths
    cd {{ client_dir_abs }} && pnpm nuxt dev

# start the server dev server
dev-s: _ensure_client_dist_dir_for_dev
    cargo run -p yomuyume

# build the client
build-c:
    #!/usr/bin/env zsh
    cd {{ client_dir_abs }} && pnpm nuxt generate
    if [ -f .nuxt/dist/client/manifest.webmanifest ]; then
        cp .nuxt/dist/client/manifest.webmanifest {{ client_dist_dir_abs }}/manifest.webmanifest
    fi

_build_blurhash_wasm:
    #!/usr/bin/env zsh
    src="{{ rust_dir }}/blurhash-webp-wasm"
    dist="{{ client_dir_abs }}/components/image/blurhash-webp-wasm"

    rm -rf "$dist"
    cd "$src"
    wasm-pack build --target web --out-dir "$dist"
    rm "$dist/.gitignore"

_build_jxl_wasm:
    #!/usr/bin/env zsh
    src="{{ rust_dir }}/jxl-webp-wasm"
    dist="{{ client_dir_abs }}/components/image/jxl-webp-wasm"

    rm -rf "$dist"
    cd "$src"
    wasm-pack build --target web --out-dir "$dist"
    rm "$dist/.gitignore"

# build the wasm packages
build-w:
    just _build_blurhash_wasm
    just _build_jxl_wasm

# build the server
build-s +args="":
    cargo build -p yomuyume --release {{ args }}

# upgrade client dependencies
upgrade-c:
    cd {{ client_dir }} && pnpm upgrade

# upgrade rust dependencies
upgrade-s +args="":
    cargo update {{ args }}

backend_constants_file := "src-rust/server/utils/constants.rs"
frontend_common_file := "src-frontend/composables/api/common.ts"

# generate API paths from backend's constants.rs file for frontend's common.ts file
gen-api-paths:
    #!/usr/bin/env python3
    import re

    def convert_line(line):
        line = line.strip()
        if not line.startswith("pub const"):
            return line

        # Extract name and value
        match = re.match(r'pub const (\w+): &str = "(.*?)";', line)
        if not match:
            return line

        name = match.group(1)
        value = match.group(2)

        # Check for path parameters (curly braces)
        path_params = re.findall(r'\{(\w+)\}', value)

        if path_params:
            # Convert path parameters to camelCase
            ts_params = []
            for param in path_params:
                camel_case_param = ''.join(word.capitalize() if i > 0 else word for i, word in enumerate(param.split('_')))
                ts_params.append(f"{camel_case_param}: string")

            # Create TypeScript arrow function
            ts_param_string = ", ".join(ts_params)
            for param in path_params:
                camel_case_param = ''.join(word.capitalize() if i > 0 else word for i, word in enumerate(param.split('_')))
                value = value.replace("{"+param+"}", "${"+camel_case_param+"}")

            return r"export const {} = ({}) => `{}`;".format(name, ts_param_string, value)
        else:
            # Create simple TypeScript constant
            return r'export const {} = "{}";'.format(name, value)

    marker = "API paths - DO NOT MODIFY THIS LINE"
    start_marker = f"// START: {marker}"
    end_marker = f"// END: {marker}"

    results = []

    with open("{{ backend_constants_file }}", "r") as infile:
        ready = False
        for line in infile:
            if line.startswith(start_marker):
                ready = True
                continue
            if line.startswith(end_marker):
                if ready == False:
                    raise Exception(f"'{start_marker}' line not found")
                ready = False
                break
            if ready:
                results.append(convert_line(line))

    if ready == True:
        raise Exception(f"'{end_marker}' line not found")

    joined = "\n".join(results)
    output_path = "{{ frontend_common_file }}"

    # Check if the markers exist in the file already
    try:
        with open(output_path, "r") as outfile:
            content = outfile.read()

        if start_marker in content and end_marker in content:
            # Replace content between markers
            pattern = re.compile(f"{start_marker}.*?{end_marker}", re.DOTALL)
            new_content = pattern.sub(f"{start_marker}\n{joined}\n{end_marker}", content)

            # Only write if content has changed to avoid unnecessary file updates
            if content != new_content:
                with open(output_path, "w") as outfile:
                    outfile.write(new_content)
                    print(f"Updated \"{output_path}\"")
            else:
                print(f"No changes needed for \"{output_path}\"")
        else:
            # Append markers and content
            with open(output_path, "w") as outfile:
                outfile.write(f"{content}\n{start_marker}\n{joined}\n{end_marker}\n")
                print(f"Updated \"{output_path}\"")
    except FileNotFoundError:
        # Create new file with markers and content
        with open(output_path, "w") as outfile:
            outfile.write(f"{start_marker}\n{joined}\n{end_marker}\n")
            print(f"Created \"{output_path}\"")

build:
    #!/usr/bin/env zsh
    just build-w
    just gen-api-paths
    just build-c
    just build-s