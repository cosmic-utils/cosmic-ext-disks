#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -gt 0 ]]; then
  exec "$@"
fi

./target/debug/cosmic-ext-storage-service &
exec cargo test -p storage-testing --test harness_smoke
