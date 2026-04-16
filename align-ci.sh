#!/bin/bash
################################################################################
#
#    Copyright (c) 2026.
#    Haixing Hu, Qubit Co. Ltd.
#
#    All rights reserved.
#
################################################################################
#
# One-shot auto-fix to match local CI (fmt + clippy on all targets, then verify).
# Run from repo root: ./align-ci.sh
#

set -e

cd "$(dirname "$0")"

echo "==> cargo fmt"
cargo fmt

echo "==> cargo clippy --fix (all targets / features)"
cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features

echo "==> cargo clippy (verify, -D warnings)"
cargo clippy --all-targets --all-features -- -D warnings

echo "Done. CI-style checks should pass; run ./ci-check.sh for the full pipeline."
