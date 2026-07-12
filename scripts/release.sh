#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# ── Options (defaults = validate + tag, nothing else) ─────────────────────
DO_TAG=1
DO_PUSH=0
DO_PUBLISH=0
ASSUME_YES=0
SKIP_TESTS=0
SKIP_VALIDATION=0
REQUIRE_CHANGELOG=0

usage() {
    cat <<'EOF'
release.sh — validate, tag, push, and publish deep-time

Reads the crate version from Cargo.toml (currently the [package] version at the
repo root). The git tag matches that version exactly, e.g. 0.1.0-beta.21.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 DEFAULT BEHAVIOR (no flags)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ./scripts/release.sh

  1. Pre-flight     clean tree, branch warning, tag/changelog checks
  2. Validation     fmt, clippy, tests, docs, examples, publish --dry-run
  3. Tag            git tag -a {version}  (asks for confirmation)

  Does NOT push or publish. Stops after creating the local tag.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 OPTIONS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Validation
  --no-tag
      Run the full validation suite but do not create a git tag.
      Use this as a pre-release dry run while you are still editing.

  --skip-tests
      Run everything except the test matrix (scripts/test-matrix.sh).
      fmt, clippy, docs, examples, and publish --dry-run still run.

  --tag-only
      Skip ALL validation. Only create the git tag (if it does not already
      exist at HEAD). Dangerous — prefer the default flow.

  --require-changelog
      Fail if CHANGELOG.md does not contain the release version string.
      Without this flag, a missing entry is a warning you can accept.

Tagging / remotes / crates.io
  --push
      After validation/tagging, push the current branch and the release tag
      to origin. Can be combined with other flags in a single invocation.

  --publish
      After validation/tagging, run `cargo publish` (needs a crates.io token).
      docs.rs builds from the uploaded crate, not from the git tag.

General
  --yes
      Skip all [y/N] prompts (branch warnings, tag, push, publish).

  -h, --help
      Show this help.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 COMMON INVOCATIONS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Check everything, no tag yet (safe while preparing a release):
    ./scripts/release.sh --no-tag

  Full local release through tagging:
    ./scripts/release.sh

  Validate, tag, push, and publish in one go:
    ./scripts/release.sh --push --publish --yes

  Push an existing tag (already created at HEAD on a previous run):
    ./scripts/release.sh --push --yes
    (validation is skipped when the tag already points at HEAD; only the
     push runs — see "Tag reuse" below)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 RECOMMENDED WORKFLOW
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Before running the script (manual steps it does not do for you):

    1. Bump version in Cargo.toml
    2. Add a CHANGELOG.md entry for that version
    3. Commit on main

  Then pick one path:

  A) Cautious (two commands)
       ./scripts/release.sh --no-tag          # full CI check, no side effects
       ./scripts/release.sh --push --yes      # tag + push when satisfied

  B) All-in-one
       ./scripts/release.sh --push --publish --yes

  The script is designed to be idempotent for tagging: if {version} already
  exists at the current commit, tag creation is skipped and later steps
  (--push) can still proceed.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 VALIDATION (what runs unless skipped)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Pre-flight
    • working tree must be clean
    • warn if not on main
    • tag {version} must not exist on a different commit
    • changelog warning (or hard fail with --require-changelog)

  Toolchain checks (MSRV read from Cargo.toml rust-version field)
    • rustfmt  (stable by default)

  Same commands as CI, plus publish --dry-run:
    • cargo fmt --all -- --check
    • ./scripts/test-matrix.sh (no-std, no-std-extended, full —
      same as CI and full-test.sh)
    • ./scripts/validate.sh (clippy + docs + examples — same as CI / full-test.sh)
    • cargo publish --dry-run

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 TAG REUSE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  If git tag {version} already points at HEAD:
    • validation is skipped (assumes a prior successful run)
    • tag creation is skipped
    • --push still pushes branch + tag

  If the tag exists on a different commit → hard error.

  Use --tag-only to force tag creation without validation (still skips if the
  tag already exists at HEAD).

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 ENVIRONMENT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  MSRV_TOOLCHAIN     Toolchain for clippy/tests/docs (default: Cargo.toml MSRV)
  RUSTFMT_TOOLCHAIN  Toolchain for rustfmt (default: stable)

  Install MSRV locally before first use:
    rustup toolchain install 1.90 --component clippy

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 PREREQUISITES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  • rustup with MSRV + stable toolchains
  • python3 (reads version/MSRV from Cargo.toml)
  • for --publish: logged in to crates.io (cargo login or CARGO_REGISTRY_TOKEN)
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --tag-only)
            SKIP_VALIDATION=1
            DO_TAG=1
            ;;
        --no-tag)
            DO_TAG=0
            ;;
        --push)
            DO_PUSH=1
            ;;
        --publish)
            DO_PUBLISH=1
            ;;
        --yes)
            ASSUME_YES=1
            ;;
        --skip-tests)
            SKIP_TESTS=1
            ;;
        --require-changelog)
            REQUIRE_CHANGELOG=1
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            echo "Run scripts/release.sh --help for usage." >&2
            exit 2
            ;;
    esac
    shift
done

log() {
    printf '\n==> %s\n' "$*"
}

run() {
    printf '+ %s\n' "$*"
    "$@"
}

confirm() {
    if [[ "$ASSUME_YES" -eq 1 ]]; then
        return 0
    fi
    local prompt="$1"
    printf '%s [y/N] ' "$prompt"
    read -r reply
    case "$reply" in
        y | Y | yes | YES) return 0 ;;
        *) return 1 ;;
    esac
}

read_version() {
    python3 - <<'PY'
import pathlib
import re

text = pathlib.Path("Cargo.toml").read_text()
match = re.search(r'^version\s*=\s*"([^"]+)"\s*$', text, re.MULTILINE)
if not match:
    raise SystemExit("Could not read version from Cargo.toml")
print(match.group(1))
PY
}

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

ensure_toolchain() {
    local toolchain="$1"
    shift

    if ! command -v rustup >/dev/null 2>&1; then
        echo "rustup is required but was not found in PATH." >&2
        echo "Install from https://rustup.rs or ensure rustup precedes other cargo installs." >&2
        exit 1
    fi

    # Use rustup's alias resolution (stable, 1.90, etc.) instead of matching
    # the full toolchain directory name (stable-x86_64-unknown-linux-gnu).
    if ! rustup which rustc --toolchain "${toolchain}" >/dev/null 2>&1; then
        echo "Missing Rust toolchain '${toolchain}'." >&2
        if [[ $# -gt 0 ]]; then
            echo "Install with: rustup toolchain install ${toolchain} $*" >&2
        else
            echo "Install with: rustup toolchain install ${toolchain}" >&2
        fi
        exit 1
    fi

    while [[ $# -gt 0 ]]; do
        if [[ "$1" == "--component" ]]; then
            shift
            local component="$1"
            if ! rustup component list --toolchain "${toolchain}" | grep -q "^${component}.*(installed)"; then
                echo "Toolchain '${toolchain}' is missing component '${component}'." >&2
                echo "Install with: rustup component add ${component} --toolchain ${toolchain}" >&2
                exit 1
            fi
        fi
        shift
    done
}

tag_exists() {
    git rev-parse "$TAG" >/dev/null 2>&1
}

# Annotated tags (git tag -a) have their own object SHA; peel to the commit.
tag_commit() {
    git rev-parse "${TAG}^{commit}"
}

tag_points_at_head() {
    tag_exists && [[ "$(tag_commit)" == "$(git rev-parse HEAD)" ]]
}

print_plan() {
    log "Release plan for deep-time ${VERSION} (tag: ${TAG}, commit: $(git rev-parse --short HEAD))"
    printf '  %-22s %s\n' "Validation:" "$([[ "$SKIP_VALIDATION" -eq 1 ]] && echo skip || echo run)"
    if [[ "$SKIP_VALIDATION" -eq 0 ]]; then
        printf '  %-22s %s\n' "  test matrix:" "$([[ "$SKIP_TESTS" -eq 1 ]] && echo skip || echo run)"
    fi
    if [[ "$DO_TAG" -eq 1 ]]; then
        if tag_points_at_head; then
            printf '  %-22s %s\n' "Tag:" "skip (already at HEAD)"
        else
            printf '  %-22s %s\n' "Tag:" "create ${TAG}"
        fi
    else
        printf '  %-22s %s\n' "Tag:" "skip (--no-tag)"
    fi
    printf '  %-22s %s\n' "Push:" "$([[ "$DO_PUSH" -eq 1 ]] && echo yes || echo no)"
    printf '  %-22s %s\n' "Publish:" "$([[ "$DO_PUBLISH" -eq 1 ]] && echo yes || echo no)"
    if [[ "$ASSUME_YES" -eq 1 ]]; then
        printf '  %-22s %s\n' "Confirmations:" "auto (--yes)"
    else
        printf '  %-22s %s\n' "Confirmations:" "interactive"
    fi
}

run_validation() {
    log "Pre-flight checks"

    if [[ -n "$(git status --porcelain)" ]]; then
        echo "Working tree is not clean. Commit or stash changes before releasing." >&2
        git status --short >&2
        exit 1
    fi

    local branch
    branch="$(git branch --show-current)"
    if [[ -z "$branch" ]]; then
        echo "Warning: detached HEAD (no branch name)." >&2
        confirm "Continue anyway?" || exit 1
    elif [[ "$branch" != "main" ]]; then
        echo "Warning: not on main (current branch: ${branch})." >&2
        confirm "Continue anyway?" || exit 1
    fi

    if tag_exists && ! tag_points_at_head; then
        echo "Git tag ${TAG} already exists on $(git rev-parse --short "${TAG}^{commit}")" >&2
        echo "but HEAD is $(git rev-parse --short HEAD)." >&2
        echo >&2
        echo "A version tag can only point at one commit. Options:" >&2
        echo "  • Release the tagged commit: git push origin HEAD && git push origin ${TAG}" >&2
        echo "  • Move tag to HEAD:            git tag -d ${TAG} && ./scripts/release.sh --push" >&2
        echo "  • New work after the tag:      bump version in Cargo.toml, commit, run again" >&2
        exit 1
    fi

    if [[ "$REQUIRE_CHANGELOG" -eq 1 ]]; then
        if ! grep -qF "${VERSION}" CHANGELOG.md 2>/dev/null; then
            echo "CHANGELOG.md does not mention version ${VERSION} (--require-changelog)." >&2
            exit 1
        fi
    elif [[ ! -s CHANGELOG.md ]] || ! grep -qF "${VERSION}" CHANGELOG.md 2>/dev/null; then
        echo "Warning: CHANGELOG.md has no entry for ${VERSION}." >&2
        confirm "Continue without a changelog entry?" || exit 1
    fi

    ensure_toolchain "$RUSTFMT" --component rustfmt
    ensure_toolchain "$MSRV" --component clippy

    log "rustfmt (${RUSTFMT})"
    run cargo "+${RUSTFMT}" fmt --all -- --check

    if [[ "$SKIP_TESTS" -eq 0 ]]; then
        log "test matrix (MSRV ${MSRV}, scripts/test-matrix.sh)"
        MSRV_TOOLCHAIN="${MSRV}" run "${ROOT}/scripts/test-matrix.sh"
    else
        log "Skipping test matrix (--skip-tests)"
    fi

    log "clippy, docs, examples (MSRV ${MSRV}, scripts/validate.sh)"
    MSRV_TOOLCHAIN="${MSRV}" run "${ROOT}/scripts/validate.sh"

    log "cargo publish --dry-run"
    run cargo publish --dry-run
}

create_tag() {
    if tag_points_at_head; then
        log "Tag ${TAG} already exists at HEAD — skipping creation"
        return 0
    fi
    if tag_exists; then
        echo "Git tag ${TAG} exists on a different commit." >&2
        exit 1
    fi
    log "Create git tag ${TAG}"
    confirm "Create annotated tag ${TAG} on $(git rev-parse --short HEAD)?" || exit 1
    run git tag -a "$TAG" -m "deep-time ${VERSION}"
    echo "Created tag ${TAG}"
}

push_release() {
    log "Push to origin"
    local branch
    branch="$(git branch --show-current)"
    if [[ -z "$branch" ]]; then
        echo "Cannot push: detached HEAD." >&2
        exit 1
    fi
    if ! tag_points_at_head; then
        echo "Tag ${TAG} is not on HEAD. Create the tag before pushing." >&2
        exit 1
    fi
    confirm "Push ${branch} and ${TAG} to origin?" || exit 1
    run git push origin HEAD
    run git push origin "$TAG"
}

publish_crate() {
    log "Publish deep-time ${VERSION} to crates.io"
    confirm "Run cargo publish for ${VERSION}?" || exit 1
    run cargo publish
    echo "Published ${VERSION} to crates.io"
    echo "docs.rs will build automatically from the crate upload."
}

# ── Main ──────────────────────────────────────────────────────────────────

VERSION="$(read_version)"
MSRV="${MSRV_TOOLCHAIN:-$(read_msrv)}"
RUSTFMT="${RUSTFMT_TOOLCHAIN:-stable}"
TAG="${VERSION}"

print_plan

if [[ "$SKIP_VALIDATION" -eq 1 ]]; then
    log "Skipping validation (--tag-only)"
elif tag_points_at_head && { [[ "$DO_TAG" -eq 1 ]] || [[ "$DO_PUSH" -eq 1 ]] || [[ "$DO_PUBLISH" -eq 1 ]]; }; then
    log "Tag ${TAG} already at HEAD — skipping validation (prior successful run assumed)"
else
    run_validation
fi

if [[ "$DO_TAG" -eq 1 ]]; then
    create_tag
else
    log "Skipping tag creation (--no-tag)"
fi

if [[ "$DO_PUSH" -eq 1 ]]; then
    push_release
fi

if [[ "$DO_PUBLISH" -eq 1 ]]; then
    publish_crate
fi

log "Done."