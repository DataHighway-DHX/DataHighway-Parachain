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
--name "Data Highway Development Chain" \
--alice \
--collator \
--force-authoring \
--chain <para chain raw chain spec> \
--base-path /tmp/parachain/alice \
--port 40333 \
--ws-port 8844 \
-- \
--execution wasm \
--chain <relay chain raw chain spec> \
--port 30343 \
--ws-port 9977
```

## Run a Collator node as a parachain to Rococo <a id="chapter-f0264f"></a>

#### Fetch repository and dependencies

* Fork and clone the repository

* Install or update Rust and dependencies. Build the WebAssembly binary from all code

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast && \
./scripts/init.sh
cargo build --release
```

#### Build runtime code

```bash
cargo build --release
```

#### Create custom chain spec

```bash
rm rococo-parachain-plain.json
rm rococo-parachain-2026-raw.json
./target/release/datahighway-collator build-spec --chain rococo --disable-default-bootnode > rococo-parachain-plain.json
./target/release/datahighway-collator build-spec --chain rococo-parachain-plain.json --raw --disable-default-bootnode > rococo-parachain-2026-raw.json

```


> Remember to purge the chain state if you change anything (database and keys)

```bash
./target/release/datahighway-collator purge-chain --chain "local" --base-path /tmp/parachain/alice

```

Or just:
```
rm -rf /tmp/parachain/alice
```

#### Start Node

Run Alice's bootnode using the raw chain definition file that was generated

```bash
./target/release/datahighway-collator \
--name "Data Highway Development Chain" \
--alice \
--collator \
--force-authoring \
--chain rococo-parachain-2026-raw.json \
--base-path /tmp/parachain/alice \
--port 40333 \
--ws-port 8844 \
-- \
--execution wasm \
--chain ./res/rococo.json \
--port 30343 \
--ws-port 9977
```
