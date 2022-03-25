#!/bin/bash

# Create `WeightInfo` implementations for all the pallets and store it in the weight module of the `integritee-runtime`.

DATAHIGHWAY_RUNTIME_WEIGHT_DIR=polkadot-parachains/datahighway-runtime/src/weights
COLLATOR=./target/release/datahighway-collator

mkdir -p $DATAHIGHWAY_RUNTIME_WEIGHT_DIR

pallets=(
    "frame_system" \
    "cumulus_pallet_parachain_system" \
    "parachain_info" \
    "pallet_balances" \
    "pallet_bounties" \
    "pallet_child_bounties" \
    "pallet_collator_selection" \
    "pallet_collective" \
    "pallet_conviction_voting" \
    "pallet_democracy" \
    "pallet_elections_phragmen" \
    "pallet_identity" \
    "pallet_membership" \
    "pallet_multisig" \
    "pallet_preimage" \
    "pallet_proxy" \
    "pallet_referenda" \
    "pallet_scheduler" \
    "pallet_timestamp" \
    "pallet_tips" \
    "pallet_treasury" \
    "pallet_utility" \
    "pallet_indices" \
    "pallet_transaction_payment" \
    "pallet_xcm" \
)

echo building runtime-benchmarking feature...
cargo build --release \
    --features runtime-benchmarks \

for pallet in ${pallets[*]}; do
    echo benchmarking pallet: "$pallet"...

    $COLLATOR \
        benchmark \
        --chain=rococo-local \
        --steps=50 \
        --repeat=20 \
        --pallet="$pallet" \
        --extrinsic="*" \
        --execution=wasm \
        --wasm-execution=compiled \
        --heap-pages=4096 \
        --output=./$DATAHIGHWAY_RUNTIME_WEIGHT_DIR/"$pallet".rs
done
