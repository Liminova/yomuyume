client_dir := "src-frontend"
client_dist_dir := client_dir + "/.output/public"

rust_dir := justfile_dir() + "/src-rust"
build_rs_file := rust_dir + "/server/build.rs"

client_dir_abs := justfile_dir() + "/" + client_dir
client_dist_dir_abs := justfile_dir() + "/" + client_dist_dir

default:
    @just --choose

_7z_ver := "2408"
_7z_location := "src-rust/server/utils/7zz"
_7z_tar_md5 := "8908df4bec189cd1f314b54724911a36"

install-7z:
    #!/usr/bin/env bash
    if [ ! -f "{{ _7z_location }}" ]; then
        rm -f /tmp/7z.tar.xz {{ _7z_location }}
        curl -L -o /tmp/7z.tar.xz https://www.7-zip.org/a/7z{{ _7z_ver }}-linux-x64.tar.xz
        if [ -f /tmp/7z.tar.xz ]; then
            if [ "$(md5sum /tmp/7z.tar.xz | awk '{print $1}')" = "{{ _7z_tar_md5 }}" ]; then
                mkdir -p /tmp/7z && tar -xvf /tmp/7z.tar.xz -C /tmp/7z
                mv /tmp/7z/7zz {{ _7z_location }}
                rm -rf /tmp/7z2408-linux-x64
            else
                echo "7z tarball does not match the expected md5"
            fi
        else
            echo "7z2408-linux-x64.tar.xz not found"
        fi
        rm -rf /tmp/7z*
    fi

dav1d_ver := "1.5.0"
dav1d_md5 := "dda9e056e8dc95471a1126308c18868d"

install-dav1d:
    #!/usr/bin/env bash
    if [ ! -d "/usr/local/lib/dav1d-{{ dav1d_ver }}" ]; then
        curl -L -o /tmp/dav1d-{{ dav1d_ver }}.tar.gz https://code.videolan.org/videolan/dav1d/-/archive/{{ dav1d_ver }}/dav1d-{{ dav1d_ver }}.tar.gz
        if [ -f /tmp/dav1d-{{ dav1d_ver }}.tar.gz ]; then
            if [ "$(md5sum /tmp/dav1d-{{ dav1d_ver }}.tar.gz | awk '{print $1}')" = "{{ dav1d_md5 }}" ]; then
                sudo mkdir -p /usr/local/lib && sudo tar -xvf /tmp/dav1d-{{ dav1d_ver }}.tar.gz -C /usr/local/lib
            else
                echo "dav1d-{{ dav1d_ver }}.tar.gz has been modified"
            fi
        else
            echo "dav1d-{{ dav1d_ver }}.tar.gz not found"
        fi
        rm -f /tmp/dav1d-{{ dav1d_ver }}.tar.gz
    fi

    # build dav1d
    if [ ! -d /usr/local/lib/dav1d-{{ dav1d_ver }}/build ]; then
        cd /usr/local/lib/dav1d-{{ dav1d_ver }}
        sudo mkdir build && cd build
        sudo meson setup --default-library=static ..
        sudo ninja
    fi

    # install dav1d
    cd /usr/local/lib/dav1d-{{ dav1d_ver }}/build && sudo ninja install

mold_ver := "2.37.1"
mold_md5 := "208254dc893403f418b3ab700a199faa"

install-mold:
    #!/usr/bin/env bash
    if [ ! -d /usr/local/cargo/mold-{{ mold_ver }}-x86_64-linux ]; then
        cd /usr/local/cargo
        curl -L -o mold.tar.gz https://github.com/rui314/mold/releases/download/v{{ mold_ver }}/mold-{{ mold_ver }}-x86_64-linux.tar.gz
        if [ "$(md5sum mold.tar.gz | awk '{print $1}')" = "{{ mold_md5 }}" ]; then
            tar -xvf mold.tar.gz
            rm -f mold.tar.gz
        else
            echo "mold tarball does not match the expected md5"
        fi
    fi

    # configure cargo to use mold
    rm -f /usr/local/cargo/config.toml
    printf "[target.x86_64-unknown-linux-gnu]\nlinker = \"clang\"\nrustflags = [\"-C\", \"link-arg=-fuse-ld=/usr/local/cargo/mold-{{ mold_ver }}-x86_64-linux/bin/mold\"]" > /usr/local/cargo/config.toml

wasmpack_ver := "0.13.1"
wasmpack_md5 := "60e58da7aac6ea343fa2500dd92ae061"

install-wasmpack:
    #!/usr/bin/env bash
    if [ ! -f /usr/local/bin/wasm-pack ]; then
        curl -L -o /tmp/wasm-pack.tar.gz https://github.com/rustwasm/wasm-pack/releases/download/v{{ wasmpack_ver }}/wasm-pack-v{{ wasmpack_ver }}-x86_64-unknown-linux-musl.tar.gz
        if [ -f /tmp/wasm-pack.tar.gz ]; then
            if [ "$(md5sum /tmp/wasm-pack.tar.gz | awk '{print $1}')" = "{{ wasmpack_md5 }}" ]; then
                tar -xvf /tmp/wasm-pack.tar.gz -C /tmp
                sudo mv /tmp/wasm-pack-v{{ wasmpack_ver }}-x86_64-unknown-linux-musl/wasm-pack /usr/local/bin
            else
                echo "wasm-pack tarball does not match the expected md5"
            fi
        else
            echo "wasm-pack.tar.gz not found"
        fi
        rm -rf /tmp/wasm-pack*
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

    # make sure `src-rust/server/build.rs` contains client_dist_dir twice
    with open("{{ build_rs_file }}", "r") as f:
        count = 0
        for line in f:
            if "{{ client_dist_dir }}" in line:
                count += 1
        if count != 2:
            raise Exception("\"{{ build_rs_file }}\" should contains \"{{ client_dist_dir }}\" twice, one for the build process, another one for importing into \"main.rs\"")

# lint the client codes
lint-c: gen-api-paths
    cd {{ client_dir_abs }} && pnpm eslint --fix --cache .

# lint the rust codes
lint-s: _ensure_client_dist_dir_for_dev
    cargo fmt && cargo clippy

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
    # cp .nuxt/dist/client/manifest.webmanifest .output/public/manifest.webmanifest

bh_wsm_dist := client_dir_abs / "components/image/blurhash-webp-wasm"
bh_wsm_src := rust_dir / "blurhash-webp-wasm"
jxl_wsm_dist := client_dir_abs / "components/image/jxl-webp-wasm"
jxl_wsm_src := rust_dir / "jxl-webp-wasm"

_build_blurhash_wasm:
    #!/usr/bin/env zsh
    rm -rf {{ bh_wsm_dist }}
    cd {{ bh_wsm_src }}
    wasm-pack build --target web --out-dir {{ bh_wsm_dist }}
    rm {{ bh_wsm_dist }}/.gitignore

_build_jxl_wasm:
    #!/usr/bin/env zsh
    rm -rf {{ jxl_wsm_dist }}
    cd {{ jxl_wsm_src }}
    wasm-pack build --target web --out-dir {{ jxl_wsm_dist }}
    rm {{ jxl_wsm_dist }}/.gitignore

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