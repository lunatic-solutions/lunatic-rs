#!/usr/bin/env bash

# publish all dependencies & sleep 45 sec between each publish for crates.io to update index
cd  "lunatic-test"
cargo publish
sleep 45
cd -
cd  "lunatic-macros"
cargo publish
sleep 45
cd -

# publish main crate
cargo publish