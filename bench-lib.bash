#!/usr/bin/env bash
# Shared setup for the benchmark and proof scripts. Source it, do not run it.

BINARYEN_VERSION=131

# `stylus/native-counter/Stylus.toml` pins a Binaryen version so that the optimisation applied at
# deploy time is the same one `cargo stylus verify` will replay. cargo-stylus refuses to build if the
# wasm-opt on PATH is a different version, so fetch the pinned one when it is missing.
ensure_binaryen() {
  local dir="$1"
  if command -v wasm-opt >/dev/null 2>&1 &&
     wasm-opt --version 2>/dev/null | grep -q "version $BINARYEN_VERSION\b"; then
    return 0
  fi
  local url="https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERSION}/binaryen-version_${BINARYEN_VERSION}-x86_64-linux.tar.gz"
  echo "fetching binaryen $BINARYEN_VERSION (pinned by Stylus.toml)"
  curl -sL -m 300 -o "$dir/binaryen.tar.gz" "$url"
  tar xzf "$dir/binaryen.tar.gz" -C "$dir"
  export PATH="$dir/binaryen-version_${BINARYEN_VERSION}/bin:$PATH"
}
