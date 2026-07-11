#!/usr/bin/env bash
# Run tests for one or all feature sets. Used by full-test.sh, release.sh, CI.
#
# Usage:
#   ./scripts/test-matrix.sh              # all entries
#   ./scripts/test-matrix.sh no-std       # one entry (see --help for the list)
#   ./scripts/test-matrix.sh --help
#
# Environment:
#   MSRV_TOOLCHAIN   Toolchain for cargo (defaults to rust-version in Cargo.toml).
#   TEST_NOCAPTURE=1 Append -- --nocapture to each cargo test invocation.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# shellcheck source=common.sh
source "${ROOT}/scripts/common.sh"

FULL_FEATURES="serde defmt physics mars parse hifitime chrono time std wire eop-tests lang sidereal-earth jiff-tz-bundle"
TDB_HI_FEATURES="tdb-hi hifitime"

MATRIX_NAMES=(
    no-std
    no-std-extended
    full
    tdb-hi
)

usage() {
    cat <<EOF
test-matrix.sh — run tests for a feature set

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 WHEN TO USE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Quick check while editing (pick the closest match):
    ./scripts/test-matrix.sh no-std          touched core / no-std code
    ./scripts/test-matrix.sh no-std-extended touched physics / mars / tdb-hi
    ./scripts/test-matrix.sh full            parse, tz, lang + release features

  Run everything (what full-test.sh and CI do):
    ./scripts/test-matrix.sh

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 ENTRIES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  no-std            bare minimum, no features
  no-std-extended   wire, mars, sidereal, physics, tdb-hi
  full              all release features incl. parse, jiff-tz-bundle, lang
  tdb-hi            high-precision TDB tests only (release, 3 test files)

  ./scripts/test-matrix.sh --list    print names only

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 SETUP / ENV
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  First time:
    rustup toolchain install 1.90 --component clippy

  MSRV_TOOLCHAIN   override toolchain (default: rust-version in Cargo.toml)
  TEST_NOCAPTURE=1   show println! output from tests
EOF
}

append_nocapture() {
    local -n _args=$1
    if [[ "${TEST_NOCAPTURE:-0}" == "1" ]]; then
        _args+=(-- --nocapture)
    fi
}

run_no_std() {
    script_log test-matrix "no-std"
    local args=(test --no-default-features --workspace)
    append_nocapture args
    script_run cargo_msrv "${args[@]}"
}

run_no_std_extended() {
    script_log test-matrix "no-std-extended"
    local args=(test --no-default-features --features "wire mars sidereal physics tdb-hi" --workspace)
    append_nocapture args
    script_run cargo_msrv "${args[@]}"
}

run_full() {
    script_log test-matrix "full (release)"
    local args=(test --release --no-default-features --features "$FULL_FEATURES" --workspace)
    append_nocapture args
    script_run cargo_msrv "${args[@]}"
}

run_tdb-hi() {
    script_log test-matrix "tdb-hi (release, scoped)"
    local args=(
        test --release --no-default-features --features "$TDB_HI_FEATURES"
        --test astropy_conversions_tests --test conversions_tests --test hifitime_tests
    )
    append_nocapture args
    script_run cargo_msrv "${args[@]}"
}

run_entry() {
    case "$1" in
        no-std) run_no_std ;;
        no-std-extended) run_no_std_extended ;;
        full) run_full ;;
        tdb-hi) run_tdb-hi ;;
        *)
            echo "Unknown test-matrix entry: $1" >&2
            echo "Valid entries: ${MATRIX_NAMES[*]}" >&2
            exit 2
            ;;
    esac
}

run_all() {
    local name
    for name in "${MATRIX_NAMES[@]}"; do
        run_entry "$name"
    done
}

main() {
    case "${1:-}" in
        -h | --help)
            usage
            ;;
        --list)
            printf '%s\n' "${MATRIX_NAMES[@]}"
            ;;
        "" | all | run-all)
            run_all
            ;;
        *)
            run_entry "$1"
            ;;
    esac
}

main "$@"