#!/bin/bash
set -e
export PATH="/home/lc/.cargo/bin:$PATH"
cd /home/lc/jw
/home/lc/.cargo/bin/dx serve --release --port 8080