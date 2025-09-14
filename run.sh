#!/bin/bash
# Set up environment
export PATH="/home/lc/.cargo/bin:$PATH"
export RUST_LOG=info
cd /home/lc/jw
/home/lc/.cargo/bin/dx serve --release --addr 127.0.0.1 --port 8080