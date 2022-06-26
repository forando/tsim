#!/bin/bash

cargo fmt
cargo fmt --all -- --check
cargo clippy -- -D warnings