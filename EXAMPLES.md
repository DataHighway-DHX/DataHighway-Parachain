# Table of contents

* [Run Collator Node with local relay chain](#chapter-f21efd)
* [Run a Collator node as a parachain to Rococo](#chapter-f0264f)

## Run Collator Node with local relay chain <a id="chapter-f21efd"></a>

### Intro

The development testnet only requires a single node to produce blocks.

### Run on Local Machine

* Use [Cumulus Workshop to connect with Local Relaychain](https://docs.substrate.io/tutorials/v3/cumulus/start-relay/)

* For Parachain use DataHighway Collator

* Fork and clone the repository

* Install or update Rust and dependencies. Build the WebAssembly binary from all code

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast && \
./scripts/init.sh && \
cargo build --release
```

* Purge the chain (remove relevant existing blockchain testnet database blocks and keys)

```bash
./target/release/datahighway-collator purge-chain --dev --base-path /tmp/parachain/alice

```

Or just:
```
rm -rf /tmp/parachain/alice
```

* Start local node

```bash
./target/release/datahighway-collator \
--name "DataHighway Development Parachain Collator Node" \
--alice \
--collator \
--force-authoring \
--chain <insert parachain raw chain spec> \
--base-path /tmp/parachain/alice \
--bootnodes <insert other existing collator bootnodes> \
--port 40333 \
--rpc-port 9933 \
--ws-port 8844 \
--unsafe-ws-external \
--unsafe-rpc-external \
--rpc-max-payload 1000 \
--rpc-cors=all \
--rpc-methods=Unsafe \
-- \
--execution wasm \
--chain <insert relay chain raw chain spec> \
--port 30343 \
--rpc-port 9943 \
--ws-port 9977
```

## Run a Collator node as a parachain to Rococo <a id="chapter-f0264f"></a>

Note: Refer to [this](https://github.com/DataHighway-DHX/documentation/blob/master/docs/tutorials/tutorials-node-polkadot-launch-datahighway-rococo-local.md) DataHighway tutorial for more information

#### Fetch repository and dependencies

* Fork and clone the repository

* Install or update Rust and dependencies. Build the WebAssembly binary from all code

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast && \
./scripts/init.sh
cargo build --release
```

#### Create custom chain spec

```bash
rm res/rococo-parachain-raw.json
./scripts/dump_wasm_state_and_spec.sh "rococo"
mv chain_dumps/rococo-parachain-raw.json res/rococo-parachain-raw.json

```

Copy the "rococo" relay chain specification into the `./res` folder of the DataHighway-Parachain directory (i.e. `./res/rococo.json`).

Since on Rococo you would likely be using a chain specification with custom keys rather than defaults like Alice, and running the node without the flag `--alice` then it is necessary to add the keys to the keystore.

```
./target/release/datahighway-collator key insert --base-path /tmp/parachain/datahighway-collator \
--chain rococo-parachain-raw.json \
--scheme sr25519 \
--suri <secret seed> \
--key-type aura
```

> Remember to purge the chain state if you change anything (database and keys)

```bash
./target/release/datahighway-collator purge-chain --chain "rococo" --base-path /tmp/parachain/alice

```

Or just:
```
rm -rf /tmp/parachain/alice
```

#### Start Node

Run Alice's bootnode using the raw chain definition file that was generated

```bash
./target/release/datahighway-collator \
--name "DataHighway Rococo Parachain Collator Node" \
--alice \
--collator \
--force-authoring \
--chain rococo-parachain-raw.json \
--base-path /tmp/parachain/datahighway-collator \
--port 40333 \
--ws-port 8844 \
-- \
--execution wasm \
--chain ./res/rococo.json \
--port 30343 \
--ws-port 9977
```
