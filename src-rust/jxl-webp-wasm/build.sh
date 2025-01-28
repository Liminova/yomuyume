cd /workspaces/yomuyume/src-server/jxl-webp-wasm
wasm-pack build --target web --out-dir ../../src/lib/jxl-webp-wasm
cd -
rm /workspaces/yomuyume/src/lib/jxl-webp-wasm/.gitignore