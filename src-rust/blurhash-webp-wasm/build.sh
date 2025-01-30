cd /workspaces/yomuyume/src-server/blurhash-webp-wasm
wasm-pack build --target web --out-dir /workspaces/yomuyume/src/components/image/blurhash-webp-wasm
cd -
rm /workspaces/yomuyume/src/components/image/blurhash-webp-wasm/.gitignore