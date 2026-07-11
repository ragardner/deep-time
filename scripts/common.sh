# Shared helpers for deep-time scripts. Source from other scripts; do not execute directly.

read_msrv() {
    python3 - <<'PY'
import pathlib
import re

text = pathlib.Path("Cargo.toml").read_text()
match = re.search(r'^rust-version\s*=\s*"([^"]+)"\s*$', text, re.MULTILINE)
if not match:
    raise SystemExit("Could not read rust-version from Cargo.toml")
print(match.group(1))
PY
}

# Pin to Cargo.toml MSRV when MSRV_TOOLCHAIN is unset.
ensure_msrv_toolchain() {
    if [[ -z "${MSRV_TOOLCHAIN:-}" ]]; then
        MSRV_TOOLCHAIN="$(read_msrv)"
        export MSRV_TOOLCHAIN
    fi
}

# Fail fast with install hints (same checks as release.sh).
ensure_msrv_toolchain_installed() {
    local components=("$@")

    ensure_msrv_toolchain

    if ! command -v rustup >/dev/null 2>&1; then
        echo "rustup is required but was not found in PATH." >&2
        echo "Install from https://rustup.rs or ensure rustup precedes other cargo installs." >&2
        exit 1
    fi

    if ! rustup which rustc --toolchain "${MSRV_TOOLCHAIN}" >/dev/null 2>&1; then
        echo "Missing Rust toolchain '${MSRV_TOOLCHAIN}' (Cargo.toml MSRV)." >&2
        echo "Install with: rustup toolchain install ${MSRV_TOOLCHAIN}" >&2
        exit 1
    fi

    local component
    for component in "${components[@]}"; do
        if ! rustup component list --toolchain "${MSRV_TOOLCHAIN}" | grep -q "^${component}.*(installed)"; then
            echo "Toolchain '${MSRV_TOOLCHAIN}' is missing component '${component}'." >&2
            echo "Install with: rustup component add ${component} --toolchain ${MSRV_TOOLCHAIN}" >&2
            exit 1
        fi
    done
}

cargo_msrv() {
    ensure_msrv_toolchain_installed
    cargo "+${MSRV_TOOLCHAIN}" "$@"
}

script_log() {
    local prefix="$1"
    shift
    printf '\n==> %s: %s\n' "$prefix" "$*"
}

script_run() {
    printf '+ %s\n' "$*"
    "$@"
}