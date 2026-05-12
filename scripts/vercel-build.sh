#!/usr/bin/env bash
set -euo pipefail

if ! command -v wasm-pack >/dev/null 2>&1; then
  cargo install wasm-pack --locked
fi

wasm-pack build wasm --target web --out-dir pkg --release
