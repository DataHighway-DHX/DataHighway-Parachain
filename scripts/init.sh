#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

rustup update stable
rustup update nightly
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup default nightly
rustup override set nightly
