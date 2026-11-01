#!/usr/bin/env bash
# The checks every commit must pass: CI's, plus the Python tests. Runs them
# all, then lists what failed and what could not run.
#
#   scripts/check.sh            the working tree
#   scripts/check.sh --staged   only what is staged, in a throwaway worktree
#                               (never `git stash`: other sessions share it)
#
# SHELLCHECK names the shellcheck binary (default: shellcheck on PATH).
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
SHELLCHECK=${SHELLCHECK:-shellcheck}
SHELLCHECK_VERSION=0.11.0 # as .github/workflows/ci.yml

failed=()
skipped=()
warnings=()

step() {
    echo "==> $*"
    if ! "$@"; then
        failed+=("$*")
    fi
}

checks() {
    cd "$1"
    step cargo fmt --check
    step cargo clippy --locked --workspace --all-targets -- -D warnings
    step cargo test --locked --workspace

    if command -v "$SHELLCHECK" >/dev/null; then
        version=$("$SHELLCHECK" --version | sed -n 's/^version: //p')
        [ "$version" = "$SHELLCHECK_VERSION" ] ||
            warnings+=("shellcheck is $version, CI uses $SHELLCHECK_VERSION: its findings may differ")
        step "$SHELLCHECK" install.sh scripts/*.sh scripts/build/*.sh cli/tests/fixtures/make-traces.sh cli/tests/scripts/*.sh
    else
        skipped+=("shellcheck: not installed ($SHELLCHECK_VERSION, as CI)")
    fi

    for test in cli/tests/scripts/test-*.sh; do
        step "$test"
    done

    has_torch=no
    python3 -I -c 'import torch' 2>/dev/null && has_torch=yes
    for test in training/test_*.py; do
        if [ "$has_torch" = no ] && grep -q '^import torch' "$test"; then
            skipped+=("$test: needs torch")
        else
            step python3 -I "$test"
        fi
    done
}

if [ "${1:-}" = --staged ]; then
    if git -C "$ROOT" diff --staged --quiet; then
        echo "nothing is staged" >&2
        exit 1
    fi
    tree=$(mktemp -d)
    git -C "$ROOT" worktree add --quiet --detach "$tree"
    trap 'git -C "$ROOT" worktree remove --force "$tree"' EXIT
    git -C "$ROOT" diff --staged --binary | git -C "$tree" apply
    checks "$tree"
elif [ $# -eq 0 ]; then
    checks "$ROOT"
else
    echo "usage: scripts/check.sh [--staged]" >&2
    exit 2
fi

echo
for item in "${warnings[@]}"; do
    echo "warning: $item"
done
for item in "${skipped[@]}"; do
    echo "not run: $item"
done
for item in "${failed[@]}"; do
    echo "FAIL     $item"
done
if [ ${#failed[@]} -gt 0 ]; then
    echo "${#failed[@]} check(s) failed" >&2
    exit 1
fi
echo "all checks passed"
