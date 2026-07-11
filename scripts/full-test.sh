#!/usr/bin/env bash
# Run everything CI runs, locally. See --help for when to use this vs the
# individual scripts (test-matrix.sh, validate.sh, release.sh).
#
# Usage:
#   ./scripts/full-test.sh
#   ./scripts/full-test.sh --help

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
    cat <<'EOF'
full-test.sh — run everything CI runs, locally

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 WHEN TO USE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Before you push or open a PR:
    ./scripts/full-test.sh

  If this passes, CI should pass.

  While coding, run one piece instead (faster):
    ./scripts/test-matrix.sh no-std       # tests only
    ./scripts/validate.sh clippy            # clippy only

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 WHAT IT RUNS (in order)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  1. cargo fmt --all                     fixes formatting (CI only checks)
  2. ./scripts/test-matrix.sh            all 4 test feature sets
  3. ./scripts/validate.sh               clippy + docs + examples

  Same commands as GitHub Actions and release.sh (except release uses fmt --check).

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 FIRST-TIME SETUP
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Install the MSRV toolchain once (version from Cargo.toml rust-version):
    rustup toolchain install 1.90 --component clippy

  More detail: ./scripts/test-matrix.sh --help
                ./scripts/validate.sh --help
                ./scripts/release.sh --help
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

log "test matrix (scripts/test-matrix.sh)"
TEST_NOCAPTURE=1 run "${ROOT}/scripts/test-matrix.sh"

log "clippy, docs, examples (scripts/validate.sh)"
run "${ROOT}/scripts/validate.sh"

log "full-test passed"
