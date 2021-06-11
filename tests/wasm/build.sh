#!/usr/bin/env bash
rustc add.rs --target=wasm32-unknown-unknown --crate-type=cdylib -C opt-level=3
rustc add_import.rs --target=wasm32-unknown-unknown --crate-type=cdylib -C opt-level=3
rustc proxy.rs --target=wasm32-unknown-unknown --crate-type=cdylib -C opt-level=3
rustc proxy_import_allow.rs --target=wasm32-unknown-unknown --crate-type=cdylib -C opt-level=3
rustc proxy_import_forbid.rs --target=wasm32-unknown-unknown --crate-type=cdylib -C opt-level=3