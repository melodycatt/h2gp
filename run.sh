#!/bin/sh
cargo build --manifest-path ~/h2gp/record_mac/Cargo.toml --release --target aarch64-apple-darwin
exec espflash flash --monitor --chip esp32c3 $1 2>&1 | ~/h2gp/record_mac/target/aarch64-apple-darwin/release/record_mac
