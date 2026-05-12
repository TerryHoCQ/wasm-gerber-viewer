#!/usr/bin/env bash
set -euo pipefail

export PATH="/rust/bin:$HOME/.cargo/bin:$PATH"
if [ -f /rust/env ]; then
  # shellcheck disable=SC1091
  . /rust/env
fi

rustup target add wasm32-unknown-unknown

if ! command -v wasm-pack >/dev/null 2>&1; then
  curl -sSf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh
  export PATH="$HOME/.cargo/bin:$PATH"
fi

cd wasm
wasm-pack build --target web --out-dir pkg --release
