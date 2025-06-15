#!/bin/sh
set -e
ARCH=$(uname -m)
if [ "$ARCH" = "x86_64" ]; then
    exec ./target/jw_code "$@"
elif [ "$ARCH" = "aarch64" ]; then
    exec ./target/jw_code_arm64 "$@"
else
    echo "Unsupported architecture: $ARCH"
    exit 1
fi
