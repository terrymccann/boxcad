#!/usr/bin/env bash
# Set the app version everywhere it is written down.
#   scripts/bump.sh 0.2.0
# package.json holds the version. The Tauri config reads it from there, and this
# script copies it to the Cargo workspace, which every crate inherits.
set -euo pipefail

version="${1:-}"
if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "usage: scripts/bump.sh MAJOR.MINOR.PATCH   (for example 0.2.0)" >&2
  exit 1
fi

cd "$(dirname "$0")/.."

# package.json and package-lock.json
npm version "$version" --no-git-tag-version --allow-same-version >/dev/null

# The single version line under [workspace.package] in Cargo.toml
perl -0pi -e "s/(\[workspace\.package\]\nversion = \")[^\"]*\"/\${1}$version\"/" Cargo.toml

# Cargo.lock records each workspace crate's version too
cargo update --workspace --offline >/dev/null 2>&1 || cargo update --workspace >/dev/null

grep -q "^version = \"$version\"$" Cargo.toml || { echo "Cargo.toml was not updated" >&2; exit 1; }
grep -q "\"version\": \"$version\"" package.json || { echo "package.json was not updated" >&2; exit 1; }
echo "Box CAD is now at $version"
