#!/bin/bash
# Wrapper: uses zig cc to cross-compile for arm-linux-gnueabihf (32-bit hard-float ARM)
# Translates Rust target triples to zig-compatible targets
args=()
for arg in "$@"; do
    case "$arg" in
        --target=armv7-unknown-linux-gnueabihf)
            args+=("--target=arm-linux-gnueabihf")
            ;;
        --target=armv7-unknown-linux-gnueabi)
            args+=("--target=arm-linux-gnueabi")
            ;;
        *)
            args+=("$arg")
            ;;
    esac
done
exec zig cc -target arm-linux-gnueabihf "${args[@]}"
