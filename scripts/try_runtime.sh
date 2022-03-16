#!/bin/bash

# Run the following after running
# ./scripts/dump_wasm_state_and_spec.sh
# and after running a collator node
DUMP_DIR=chain_dumps
COLLATOR=./target/release/datahighway-collator
TRY_RUNTIME_LOGS=logs-try-runtime
mkdir -p $TRY_RUNTIME_LOGS

echo try-runtime...

./scripts/init.sh

cargo build --release \
--features try-runtime \

RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
RUST_BACKTRACE=1 \
$COLLATOR \
try-runtime \
# --chain=${DUMP_DIR}/rococo-local-raw.json \
--chain=./rococo-local-parachain-2000-raw.json \
--url wss://127.0.0.1:9933 \ # default is 9944
--block-at 1 \
on-runtime-upgrade \
live
