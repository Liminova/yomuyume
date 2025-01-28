cd /workspaces/yomuyume/src-server/blurhash-webp-wasm
wasm-pack build --target web --out-dir ../../src/lib/blurhash-webp-wasm
cd -
rm /workspaces/yomuyume/src/lib/blurhash-webp-wasm/.gitignore