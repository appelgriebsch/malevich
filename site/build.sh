#!/bin/sh
# Build the site into site/dist: the generator first (pages and figures), then
# the wasm the live pages use. Serve with: python3 -m http.server 4173 --directory site/dist
set -eu
root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
out="$root/site/dist"
cd "$root"
cargo run --release -q -p malevich-site -- --out "$out"
cd "$root/js"
wasm-pack build native --target web --out-dir "$out/wasm" --release
rm -f "$out/wasm/.gitignore" "$out/wasm/README.md" "$out/wasm/.npmignore"
# The glue is ESM; GitHub Pages must not treat it as CommonJS.
printf '%s\n' '{"type":"module"}' > "$out/wasm/package.json"
echo "built $out"
