#!/bin/bash

# Create `WeightInfo` implementations for all the pallets and store it in the weight module of the `integritee-runtime`.

DATAHIGHWAY_RUNTIME_WEIGHT_DIR=polkadot-parachains/datahighway-runtime/src/weights
COLLATOR=./target/release/datahighway-collator

mkdir -p $DATAHIGHWAY_RUNTIME_WEIGHT_DIR

pallets=(
  "frame_system" \
  "cumulus_pallet_parachain_system" \
  "pallet_utility" \
  "pallet_randomness_collective_flip" \
  "pallet_timestamp" \
  "pallet_identity" \
  "pallet_recovery" \
  "pallet_scheduler" \
  "pallet_sudo" \
  "parachain_info" \
  "pallet_indices" \
  "pallet_balances" \
  "pallet_transaction_payment" \
  "pallet_authorship" \
  "pallet_collator_selection" \
  "pallet_session" \
  "pallet_democracy" \
  "pallet_aura" \
  "cumulus_pallet_aura_ext" \
  "cumulus_pallet_xcmp_queue" \
  "pallet_xcm" \
  "cumulus_pallet_xcm" \
  "cumulus_pallet_dmp_queue" \
  "pallet_collective::<Instance1>" \
  "pallet_collective::<Instance2>" \
  "pallet_elections_phragmen" \
  "pallet_membership::<Instance1>" \
  "pallet_treasury" \
  "pallet_bounties" \
  "pallet_child_bounties" \
  "pallet_tips" \
  "pallet_preimage" \
  "pallet_proxy" \
  "pallet_multisig" \
  "pallet_referenda" \
  "pallet_conviction_voting" \
  "membership_supernodes" \
  "roaming_operators" \
  "roaming_networks" \
  "roaming_organizations" \
  "roaming_network_servers" \
  "roaming_devices" \
  "roaming_routing_profiles" \
  "roaming_service_profiles" \
  "roaming_accounting_policies" \
  "roaming_agreement_policies" \
  "roaming_network_profiles" \
  "roaming_device_profiles" \
  "roaming_sessions" \
  "roaming_billing_policies" \
  "roaming_charging_policies" \
  "roaming_packet_bundles" \
  "mining_setting_token" \
  "mining_setting_hardware" \
  "mining_rates_token" \
  "mining_rates_hardware" \
  "mining_sampling_token" \
  "mining_sampling_hardware" \
  "mining_eligibility_token" \
  "mining_eligibility_hardware" \
  "mining_eligibility_proxy" \
  "mining_claims_token" \
  "mining_claims_hardware" \
  "mining_execution_token" \
  "exchange_rate" \
)

for pallet in ${pallets[*]}; do
  echo benchmarking "$pallet"...

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
