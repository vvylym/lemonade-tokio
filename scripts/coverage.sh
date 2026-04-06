#!/usr/bin/env bash
# Line coverage for the workspace with cargo-llvm-cov.
# Enforces >= 85% on the measured scope (see README "Coverage gate").
#
# Local: ./scripts/coverage.sh
# CI:    COVERAGE_CI=1 ./scripts/coverage.sh  (writes .github/lcov.info for Codecov)
set -euo pipefail

EXCLUDE='(lemonade-worker-[^/]+/|lemonade/src/(main|lib|handlers)\.rs|lemonade-service/src/(error_response|lib)\.rs)'

if [[ "${COVERAGE_CI:-}" == 1 ]]; then
  cargo llvm-cov --workspace --all-targets --all-features \
    --ignore-filename-regex="$EXCLUDE" \
    --fail-under-lines 85 \
    --lcov --output-path .github/lcov.info
else
  cargo llvm-cov --workspace --all-targets --all-features \
    --ignore-filename-regex="$EXCLUDE" \
    --fail-under-lines 85 \
    --summary-only
fi
