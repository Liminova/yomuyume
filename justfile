client_dir := justfile_dir() + "/src-frontend"
rust_dir := justfile_dir() + "/src-rust"

default:
    @just --choose

# lint the client codes
lint-c: gen-api-paths
    cd {{client_dir}} && pnpm eslint --fix --cache .

# lint the rust codes
lint-s:
    cargo fmt && cargo clippy

# start the client dev server
dev-c: gen-api-paths
    cd {{client_dir}} && pnpm nuxt dev

# start the server dev server
dev-s:
    cargo run -p yomuyume

# build the client
build-c: gen-api-paths
    #!/usr/bin/env zsh
    cd {{client_dir}} && pnpm nuxt generate
    cp .nuxt/dist/client/manifest.webmanifest .output/public/manifest.webmanifest

bh_wsm_dist := client_dir / "components/image/blurhash-webp-wasm"
bh_wsm_src := rust_dir / "blurhash-webp-wasm"
jxl_wsm_dist := client_dir / "components/image/jxl-webp-wasm"
jxl_wsm_src := rust_dir / "jxl-webp-wasm"

_build_blurhash_wasm:
    #!/usr/bin/env zsh
    rm -rf {{bh_wsm_dist}}
    cd {{bh_wsm_src}}
    wasm-pack build --target web --out-dir {{bh_wsm_dist}}
    rm {{bh_wsm_dist}}/.gitignore

_build_jxl_wasm:
    #!/usr/bin/env zsh
    rm -rf {{jxl_wsm_dist}}
    cd {{jxl_wsm_src}}
    wasm-pack build --target web --out-dir {{jxl_wsm_dist}}
    rm {{jxl_wsm_dist}}/.gitignore

# build the wasm packages
build-w:
    just _build_blurhash_wasm
    just _build_jxl_wasm

# build the server
build-s +args="":
    cargo build -p yomuyume --release {{args}}

# upgrade client dependencies
upgrade-c:
    cd {{client_dir}} && pnpm upgrade

# upgrade rust dependencies
upgrade-s +args="":
    cargo update {{args}}

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
