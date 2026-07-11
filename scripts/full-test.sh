#!/usr/bin/env bash
# Full local validation: default tests, release full-feature tests, clippy, docs, and examples.
#
# Usage:
#   ./scripts/full-test.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

FULL_FEATURES="serde physics mars parse hifitime chrono time std wire eop-tests lang sidereal-earth jiff-tz"
TDB_HI_FEATURES="tdb_hi hifitime"
TDB_HI_TESTS="--test astropy_conversions_tests --test conversions_tests --test hifitime_tests"

usage() {
    cat <<'EOF'
full-test.sh — run the full deep-time local validation suite

  ./scripts/full-test.sh

Runs, in order:

  1. cargo fmt --all
  2. cargo test --workspace
  3. cargo test --release --no-default-features --features "<full>" --workspace -- --nocapture
  4. cargo test --release --no-default-features --features "tdb_hi hifitime" <tdb-hi tests> -- --nocapture
  5. cargo clippy --workspace --all-features --all-targets
  6. cargo doc --all-features --no-deps    (your crate only; see CI)
  7. cargo run --example precision_control
  8. cargo run --example sidereal_time --features "sidereal-earth,eop,std"
  9. cargo run --example readme --features "parse,jiff-tz,euro"

Uses your active cargo/rustc toolchain. For MSRV-pinned checks, see scripts/release.sh.
EOF
}

log() {
    printf '\n==> %s\n' "$*"
}

run() {
    printf '+ %s\n' "$*"
    "$@"
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    exit 0
fi

log "full-test (commit: $(git rev-parse --short HEAD 2>/dev/null || echo unknown))"

log "cargo fmt"
run cargo fmt --all

log "cargo test (default)"
run cargo test --workspace

log "cargo test --release (full features, --nocapture)"
run cargo test --release --no-default-features --features "$FULL_FEATURES" --workspace -- --nocapture

log "cargo test --release (tdb_hi + hifitime, scoped tests, --nocapture)"
run cargo test --release --no-default-features --features "$TDB_HI_FEATURES" \
    $TDB_HI_TESTS -- --nocapture

log "cargo clippy (all features, all targets)"
run cargo clippy --workspace --all-features --all-targets -- \
    -D warnings -W clippy::collapsible_else_if

log "cargo doc (all features, --no-deps)"
run cargo doc --all-features --no-deps

log "examples"
run cargo run --example precision_control
run cargo run --example sidereal_time --features "sidereal-earth,eop,std"
run cargo run --example readme --features "parse,jiff-tz,euro"

log "full-test passed"