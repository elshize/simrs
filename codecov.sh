#!/bin/bash

set +xe

REQUIRED=100

cargo clean
RUSTFLAGS="-Zinstrument-coverage" cargo test --lib --no-run
tests="target/debug/deps/$(ls target/debug/deps | grep -e "^simrs-[a-z0-9]*$")"
$tests
cat default.profraw > all.profraw

RUSTFLAGS="-Zinstrument-coverage" cargo build --examples
objects=""
examples="$(ls target/debug/examples | grep -e ".*-[a-z0-9]*$")"
for example in $examples; do
    ./target/debug/examples/$example 2>&1 > /dev/null
    objects="$objects -object ./target/debug/examples/$example"
    cat default.profraw >> all.profraw
done

llvm-profdata merge -sparse all.profraw -o default.profdata
if ! [[ $1 = "--check" ]]; then
    llvm-cov report -Xdemangler=rustfilt $tests $objects -instr-profile=default.profdata
    llvm-cov show -Xdemangler=rustfilt $tests $objects -instr-profile=default.profdata \
        -show-line-counts-or-regions \
        -format=html > cov.html
else
    llvm-cov export -Xdemangler=rustfilt $tests $objects -instr-profile=default.profdata \
        > lcov.json
    percent=$(jq '.data[].totals.lines.percent' lcov.json)
    if (( percent < $REQUIRED )); then
        echo "Required ${REQUIRED}% line coverage. Coverage detected: $percent"
        exit 1
    else
        echo "Success!"
    fi
fi
