#!/bin/bash

set +xe

REQUIRED=100

LLVM_COV=${LLVM_COV:-llvm-cov}
LLVM_PROFDATA=${LLVM_PROFDATA:-llvm-profdata}
export LLVM_PROFILE_FILE="coverage-%p-%m.profraw"
export RUSTFLAGS="-Zinstrument-coverage"

format="$1"
[[ "$format" == "html" ]] && output="./coverage" || output="./coverage/lcov.info"

cargo clean
cargo test
cargo run --example simulation
rm -rf "$output"
grcov . --binary-path ./target/debug/ -s . -t "$format" --branch --ignore-not-existing \
    --ignore "*cargo*" \
    --ignore "*example*" \
    -o "$output"
