# Table of contents

* [Pull Requests](#chapter-4a9b69)
* [Continuous Integration](#chapter-7a8301)
* [Linting](#chapter-c345d7)
* [Debugging](#chapter-93c645)
* [Testing](#chapter-e146ec)
* [Benchmarking](#chapter-6c1b24)
* [Try-Runtime](#chapter-397b84)
* [Memory Profiling](#chapter-585a25)
* [Code Editor Configuration](#chapter-d5a9de)
* [Create new runtime modules](#chapter-18873f)
* [FAQ](#chapter-f078a2)
* [Technical Support](#chapter-c00ab7)

Note: Generate a new chapter with `openssl rand -hex 3`

## Pull Requests <a id="chapter-4a9b69"></a>

All Pull Requests should first be made into the 'main' branch.
In future the Github Actions CI badge build status that is shown in the README may depend on the outcome of building Pull Requests from a branch.

### Skipping CI

To skip running the CI unnecessarily for simple changes such as updating the documentation, include `[ci skip]` or `[skip ci]` in your Git commit message.

### Linting

Check with Rust Format. Note: If you need a specific version of it replace `+nightly` with say `+nightly-2022-03-16`
```
cargo +nightly fmt --all -- --check
```

If you wish to apply Rust Format on your changes prior to creating a PR. See [Linting](#chapter-c345d7).

```bash
cargo +nightly fmt --all
```

Optionally run Clippy

```bash
cargo clippy --release -- -D warnings
```

Optionally run check
```
cargo check
```

## Debugging <a id="chapter-93c645"></a>

### Simple Debugging

**TODO** - Replace with use of log::debug with native::debug. See https://github.com/DataHighway-DHX/node/issues/41

* Add to Cargo.toml of runtime module:
```yaml
...
    'log/std',
...
[dependencies.log]
version = "0.4.8"
```

* Add to my-module/src/lib.rs
```rust
use log::{error, info, debug, trace};
...
log::debug!("hello {:?}", world); // Only shows in terminal in debug mode
log::info!("hello {:?}", world); // Shows in terminal in release mode
```

Note: The use of `debug::native::info!("hello {:?}", world);` does not appear to work anymore since Substrate updates in Feb 2021.

### Detailed Debugging

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/datahighway ... \
  ... \
  -lruntime=debug
```

Refer to Susbtrate Debugging documentation [here](https://docs.substrate.io/v3/runtime/debugging/)

## Testing <a id="chapter-e146ec"></a>

### Run All Tests

```bash
cargo test -p datahighway-parachain-runtime
```

### Run Integration Tests Only

```
cargo test -p datahighway-parachain-runtime
```

#### Run Specific Integration Tests

Example
```
cargo test -p datahighway-parachain-runtime --test <INSERT_INTEGRATION_TEST_FILENAME>
```

## Benchmarking <a id="chapter-6c1b24"></a>

Run the following:
```
./scripts/benchmark_all_pallets.sh
```

## Try-Runtime <a id="chapter-397b84"></a>

* Run Collator nodes

* Build whilst specifying the `try-runtime` feature
```
cargo build --release features=try-runtime
```

* Run Try-Runtime so `on-runtime-upgrade` will invoke all  `OnRuntimeUpgrade` hooks in pallets and the runtime
```
RUST_LOG=runtime=trace,try-runtime::cli=trace,executor=trace \
./target/release/datahighway-collator \
try-runtime \
--chain <chain-spec> \
--execution Wasm \
--wasm-execution Compiled \
--uri <ws/s port>
--block-at <block-hash> \
on-runtime-upgrade \
live
```

Notes:
* Ensure that the Collator node was run with:
```
--rpc-max-payload 1000 \
--rpc-cors=all \
```
* The `--chain` argument must be provided
* Provide a `--uri` and `--block-at` hash from the testnet where the Collator node was launched. The defaults are the wss://127.0.0.1:9944 port and the latest finalized block respectively.
* `live` means we are going to scrape a live testnet, as opposed to loading a saved file.

References:
* https://docs.substrate.io/how-to-guides/v3/tools/try-runtime/
* https://docs.substrate.io/v3/tools/try-runtime/

## Memory Profiling <a id="chapter-585a25"></a>

```
curl -L https://github.com/koute/memory-profiler/releases/download/0.6.1/memory-profiler-x86_64-unknown-linux-gnu.tgz -o memory-profiler-x86_64-unknown-linux-gnu.tgz
tar -xf memory-profiler-x86_64-unknown-linux-gnu.tgz

export MEMORY_PROFILER_LOG=info
export MEMORY_PROFILER_LOGFILE=profiling_%e_%t.log
export MEMORY_PROFILER_OUTPUT=profiling_%e_%t.dat
export MEMORY_PROFILER_CULL_TEMPORARY_ALLOCATIONS=1
```

It should only be run on a testnet. See https://github.com/paritytech/subport/issues/257.
Purge local chain from previous tests, then:
```
LD_PRELOAD=<INSERT_PATH_TO_MEMORY_PROFILER>/libmemory_profiler.so \
./target/release/datahighway-collator <INSERT_TESTNET_ARGS>
```

```
./memory-profiler-cli server *.dat
```

View output at http://localhost:8080/

Reference:
* https://docs.substrate.io/v3/tools/memory-profiling/

## Continuous Integration <a id="chapter-7a8301"></a>

Github Actions are used for Continuous Integration.
View the latest [CI Build Status](https://github.com/DataHighway-DHX/node/actions?query=branch%3Adevelop) of the 'develop' branch, from which all Pull Requests are made into the 'master' branch.

Note: We do not watch Pull Requests from the 'master' branch, as they would come from Forked repos.

Reference: https://help.github.com/en/actions/configuring-and-managing-workflows/configuring-a-workflow

## Linting<a id="chapter-c345d7"></a>

### Clippy

#### Run Manually

##### Stable
```rust
cargo clippy --release -- -D warnings
```

##### Nightly

The following is a temporary fix. See https://github.com/rust-lang/rust-clippy/issues/5094#issuecomment-579116431

```
rustup component add clippy --toolchain nightly-2020-12-12-x86_64-unknown-linux-gnu
rustup component add clippy-preview --toolchain nightly-2020-12-12-x86_64-unknown-linux-gnu
cargo +nightly-2020-12-12 clippy-preview -Zunstable-options
```

#### Clippy and Continuous Integration (CI)

Clippy is currently disabled in CI for the following reasons.

A configuration file clippy.toml to accept or ignore different types of Clippy errors
is not available (see https://github.com/rust-lang/cargo/issues/5034). So it
currenty takes a long time to manually ignore each type of Clippy error in each file.

To manually ignore a clippy error it is necessary to do the following,
where `redundant_pattern_matching` is the clippy error type in this example:

```rust
#![cfg_attr(feature = "cargo-clippy", allow(clippy::redundant_pattern_matching))]
```

### Rust Format

[RustFmt](https://github.com/rust-lang/rustfmt) should be used for styling Rust code.
The styles are defined in the rustfmt.toml configuration file, which was generated by running `rustfmt --print-config default rustfmt.toml` and making some custom tweaks according to https://rust-lang.github.io/rustfmt/

#### Install RustFmt

```bash
rustup component add rustfmt --toolchain nightly-2020-12-12-x86_64-unknown-linux-gnu
```

#### Check Formating Changes that RustFmt before applying them

Check that you agree with all the formating changes that RustFmt will apply to identify anything that you do not agree with.

```bash
cargo +nightly fmt --all -- --check
```

#### Apply Formating Changes

```bash
cargo +nightly fmt --all
```

## Code Editor Configuration <a id="chapter-d5a9de"></a>

### Add Vertical Rulers in VS Code

Add the following to settings.json `"editor.rulers": [80,120]`, as recommended here https://stackoverflow.com/a/45951311/3208553

### EditorConfig

Install an [EditorConfig Plugin](https://editorconfig.org/) for your code editor to detect and apply the configuration in .editorconfig.

### Create new runtime modules <a id="chapter-18873f"></a>

```bash
substrate-module-new <module-name> <author>
```

## FAQ <a id="chapter-f078a2"></a>

The latest FAQ is still recorded on the DataHighway standalone codebase [here](https://github.com/DataHighway-DHX/node/blob/master/CONTRIBUTING.md#faq-), or modified in subsequent PRs.
It will be migrated into the DataHighway/documentation codebase.

## Technical Support <a id="chapter-c00ab7"></a>

* [Discord Chat](https://discord.gg/UuZN2tE)

* [Twitter](https://twitter.com/DataHighway_DHX)
