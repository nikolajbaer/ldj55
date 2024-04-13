#!/bin/sh
cargo build --target x86_64-pc-windows-gnu &&
#cargo run --features bevy/dynamic_linking --target x86_64-pc-windows-gnu
exec target/x86_64-pc-windows-gnu/debug/ld55.exe "$@"
