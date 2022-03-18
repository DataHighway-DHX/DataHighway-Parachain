#!/bin/bash

# Credits: https://github.com/integritee-network/parachain/blob/master/scripts/dump_wasm_state_and_spec.sh
#
# Helper script to generate the wasm, state and chain-spec/ -raw.json for a given chain-spec.
#
# Usage: ./scripts/dump_wasm_state_and_spec.sh <chain-spec> <para-id> <collator-binary> <dump-dir>
#
# Example: ./scripts/dump_wasm_state_and_spec.sh shell-kusama-local-dev 2000 collator ./dump_dir
#
# chain-spec is mandatory, the rest is optional.


CHAIN_SPEC=$1
COLLATOR=${2:-./target/release/datahighway-collator}
DUMP_DIR=${3:-./chain_dumps}

mkdir -p ${DUMP_DIR}

echo "dumping spec for: $CHAIN_SPEC"
echo "collator:         ${COLLATOR}"
echo "dump_dir:         ${DUMP_DIR}"
echo ""

$COLLATOR build-spec --chain ${CHAIN_SPEC} --disable-default-bootnode > $DUMP_DIR/${CHAIN_SPEC}-parachain-plain.json
$COLLATOR build-spec --chain $DUMP_DIR/${CHAIN_SPEC}-parachain-plain.json --raw --disable-default-bootnode > $DUMP_DIR/${CHAIN_SPEC}-parachain-raw-unsorted.json
jq --sort-keys . $DUMP_DIR/${CHAIN_SPEC}-parachain-raw-unsorted.json > $DUMP_DIR/${CHAIN_SPEC}-parachain-raw.json

$COLLATOR export-genesis-state --chain $DUMP_DIR/${CHAIN_SPEC}-parachain-raw.json > $DUMP_DIR/${CHAIN_SPEC}-parachain-state
$COLLATOR export-genesis-wasm --chain $DUMP_DIR/${CHAIN_SPEC}-parachain-raw.json > $DUMP_DIR/${CHAIN_SPEC}-parachain-wasm
