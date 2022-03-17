#!/bin/bash

# Create `WeightInfo` implementations for all the pallets and store it in the weight module of the `integritee-runtime`.

DATAHIGHWAY_RUNTIME_WEIGHT_DIR=polkadot-parachains/datahighway-runtime/src/weights
COLLATOR=./target/release/datahighway-collator

mkdir -p $DATAHIGHWAY_RUNTIME_WEIGHT_DIR

pallets=(
    "frame_system" \
    "pallet_utility" \
    "pallet_timestamp" \
    "pallet_indices" \
    "pallet_balances" \
    "pallet_collator_selection" \
    "pallet_treasury" \
)

for pallet in ${pallets[*]}; do
    echo benchmarking "$pallet"...

    cargo build --release \
    --features runtime-benchmarks \

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
    --output=./$DATAHIGHWAY_RUNTIME_WEIGHT_DIR/"$pallet".rs \
done
