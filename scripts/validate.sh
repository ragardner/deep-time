#!/usr/bin/env bash
# Clippy, docs, and examples — same checks as CI. Used by full-test.sh, release.sh.
#
# Usage:
#   ./scripts/validate.sh           # all checks
#   ./scripts/validate.sh clippy    # one check (see --help)
#   ./scripts/validate.sh --help
#
# Environment:
#   MSRV_TOOLCHAIN   Toolchain for cargo (defaults to rust-version in Cargo.toml).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# shellcheck source=common.sh
source "${ROOT}/scripts/common.sh"

CHECK_NAMES=(
    clippy
    docs
    examples
)

usage() {
    cat <<EOF
validate.sh — clippy, docs, and examples (same as CI)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 WHEN TO USE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  After tests pass, or when you only changed docs / examples:
    ./scripts/validate.sh              everything
    ./scripts/validate.sh clippy       lint only
    ./scripts/validate.sh docs         doc build (no-default-features + all-features)
    ./scripts/validate.sh examples     run the library examples

  Or let full-test.sh run this for you:
    ./scripts/full-test.sh

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 SETUP / ENV
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  First time:
    rustup toolchain install 1.90 --component clippy

  MSRV_TOOLCHAIN   override toolchain (default: rust-version in Cargo.toml)
EOF
}

run_clippy() {
    script_log validate "clippy (MSRV ${MSRV_TOOLCHAIN:-auto})"
    ensure_msrv_toolchain_installed clippy
    script_run cargo_msrv clippy --workspace --all-features --all-targets -- \
        -D warnings -W clippy::collapsible_else_if
}

run_docs() {
    export RUSTDOCFLAGS="-D warnings"
    script_log validate "docs — no-default-features (MSRV ${MSRV_TOOLCHAIN:-auto})"
    script_run cargo_msrv doc --no-default-features --no-deps
    script_log validate "docs — all-features (MSRV ${MSRV_TOOLCHAIN:-auto})"
    script_run cargo_msrv doc --all-features --no-deps
}

run_examples() {
    script_log validate "examples (MSRV ${MSRV_TOOLCHAIN:-auto})"
    script_run cargo_msrv run --example precision_control
    script_run cargo_msrv run --example sidereal_time --features "sidereal-earth,eop,std"
    script_run cargo_msrv run --example proper_time_path --features physics
    script_run cargo_msrv run --example readme --features "parse,jiff-tz,euro"
}

run_entry() {
    case "$1" in
        clippy) run_clippy ;;
        docs) run_docs ;;
        examples) run_examples ;;
        *)
            echo "Unknown validate entry: $1" >&2
            echo "Valid entries: ${CHECK_NAMES[*]}" >&2
            exit 2
            ;;
    esac
}

run_all() {
    local name
    for name in "${CHECK_NAMES[@]}"; do
        run_entry "$name"
    done
}

main() {
    case "${1:-}" in
        -h | --help)
            usage
            ;;
        --list)
            printf '%s\n' "${CHECK_NAMES[@]}"
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