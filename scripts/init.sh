#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

rustup update stable
rustup update nightly-2022-02-23
rustup toolchain install nightly-2022-02-23
rustup target add wasm32-unknown-unknown --toolchain nightly-2022-02-23
rustup default nightly-2022-02-23
rustup override set nightly-2022-02-23
