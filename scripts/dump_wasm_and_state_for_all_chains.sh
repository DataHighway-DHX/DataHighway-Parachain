#!/bin/bash

# Credits: https://github.com/integritee-network/parachain/blob/master/scripts/dump_wasm_and_state_for_all_chains.sh

# Remark: wasm and state will be identical for different relay-chains

# Helper script to generate the wasm, state and chain-spec/ -raw.json for a all chain-specs.
#
# Usage: ./scripts/dump_wasm_and_state_for_all_chains.sh <para-id> <collator-binary> <dump-dir>
#
# Example: ./scripts/dump_wasm_state_and_spec.sh 2000 ./collator ./dump_dir
#
# All arguments are optional.

COLLATOR=${2:-./target/release/datahighway-collator}
DUMP_DIR=${3:-./chain_dumps}

mkdir -p ${DUMP_DIR}

# Note: chachacha and polkadot have been omitted
chainspecs=(
    "rococo-dev" \
    "rococo-local" \
    "rococo" \
    "kusama" \
    "kusama-dev" \
    "kusama-local" \
    "kusama" \
)

$COLLATOR --version
# Print array values in  lines
for spec in ${chainspecs[*]}; do
  ./scripts/dump_wasm_state_and_spec.sh ${spec} ${COLLATOR} ${DUMP_DIR}
done
