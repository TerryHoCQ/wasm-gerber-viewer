#!/usr/bin/env bash
set -euo pipefail

export CARGO_HOME="/rust"
export RUSTUP_HOME="/rust"
export PATH="$CARGO_HOME/bin:$PATH"

# shellcheck disable=SC1091
. /rust/env

rustup toolchain install stable --profile minimal
rustup default stable
rustup target add wasm32-unknown-unknown --toolchain stable

if ! command -v wasm-pack >/dev/null 2>&1; then
  cargo install wasm-pack --locked
fi

cd wasm
wasm-pack build --target web --out-dir pkg --release
