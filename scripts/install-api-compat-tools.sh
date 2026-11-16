#!/usr/bin/env bash
# Install what scripts/api_compat.py needs on a Linux x86_64 runner: the pinned
# cargo-semver-checks release binary (checksum-verified) and the rustdoc JSON nightly
# (DOCS_TOOLCHAIN in the Makefile). Locally: `cargo install --locked cargo-semver-checks@<v>`.
set -euo pipefail
cd "$(dirname "$0")/.."

version="$(sed -n 's/^SEMVER_CHECKS_VERSION = "\(.*\)"$/\1/p' scripts/api_compat.py)"
case "$version" in
  0.51.0) sha256=cacadafaa5cd27ae6d05a74a103266917ebab4c5e43c898360e177d4fbf06a65 ;;
  *)
    echo "::error::No checksum for cargo-semver-checks ${version:-?} in $0; add it when bumping SEMVER_CHECKS_VERSION."
    exit 1
    ;;
esac

asset="cargo-semver-checks-x86_64-unknown-linux-gnu.tar.gz"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
curl -sSfL --retry 3 -o "$tmp/$asset" \
  "https://github.com/obi1kenobi/cargo-semver-checks/releases/download/v${version}/${asset}"
echo "${sha256}  $tmp/$asset" | sha256sum -c -
mkdir -p "$HOME/.cargo/bin"
tar -xzf "$tmp/$asset" -C "$HOME/.cargo/bin" cargo-semver-checks
cargo semver-checks --version

toolchain="$(make --no-print-directory print-docs-toolchain)"
rustup toolchain install "$toolchain" --profile minimal
