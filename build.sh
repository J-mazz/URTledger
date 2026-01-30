#!/usr/bin/env bash
set -euo pipefail

echo "Building project (host)..."
cargo build --release

echo "Done. Run target/debug/dynamic_inventory_engine for a quick check."
