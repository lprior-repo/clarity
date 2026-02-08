#!/usr/bin/env bash
# PGO Workload Script for Clarity
#
# This script runs a representative workload to generate profiling data
# for Profile-Guided Optimization (PGO).

set -euo pipefail

echo "=== PGO Workload Script ==="
echo "Running representative workload for PGO profiling..."
echo ""

# Create PGO data directory
mkdir -p pgo-data

echo "Running test suite with instrumentation..."
cargo test --workspace --lib --no-fail-fast || true

echo "=== PGO Workload Complete ==="
echo "Profile data should be in pgo-data/"
