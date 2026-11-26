#!/bin/sh
set -eu
repo_dir=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
if [ "$#" -ne 1 ]; then
  printf '%s\n' 'Usage: scripts/package-claude-skill.sh /absolute/output.tar.gz' >&2
  exit 2
fi
case "$1" in /*) output=$1 ;; *) printf '%s\n' 'Output path must be absolute.' >&2; exit 2 ;; esac
if [ -e "$output" ]; then printf '%s\n' 'Refusing to overwrite an existing archive.' >&2; exit 1; fi
cargo build --release --locked --manifest-path "$repo_dir/Cargo.toml" --target-dir "$repo_dir/target"
stage_dir=$(mktemp -d)
trap 'rm -rf "$stage_dir"' EXIT HUP INT TERM
cp -R "$repo_dir/skills/project-adviser" "$stage_dir/project-adviser"
cp "$repo_dir/target/release/stepwise" "$stage_dir/project-adviser/scripts/stepwise"
cp "$repo_dir/LICENSE" "$stage_dir/project-adviser/LICENSE"
chmod +x "$stage_dir/project-adviser/scripts/stepwise" "$stage_dir/project-adviser/scripts/stepwise.sh"
rustc -vV > "$stage_dir/project-adviser/BUILD.txt"
tar -czf "$output" -C "$stage_dir" project-adviser
printf 'Packaged skill for the current machine architecture: %s\n' "$output"
