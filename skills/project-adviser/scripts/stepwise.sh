#!/bin/sh
set -eu
skill_dir=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
if [ -x "$skill_dir/scripts/stepwise" ]; then
  exec "$skill_dir/scripts/stepwise" "$@"
fi
repo_dir=$(CDPATH='' cd -- "$skill_dir/../.." && pwd)
if [ -x "$repo_dir/target/release/stepwise" ]; then
  exec "$repo_dir/target/release/stepwise" "$@"
fi
printf '%s\n' 'The Rust helper has not been built for this installation.' >&2
printf 'Run: cargo build --release --locked --manifest-path "%s/Cargo.toml"\n' "$repo_dir" >&2
exit 1
