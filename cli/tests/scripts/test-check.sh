#!/usr/bin/env bash
# scripts/check.sh in a fake repository, with fake cargo, shellcheck and
# python3 that log their calls: it runs every check, reports failures and
# what it could not run, and --staged checks only the staged change. No
# network.
#
#   cli/tests/scripts/test-check.sh
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

# The outer environment never reaches the script under test.
unset SHELLCHECK HAS_TORCH FAIL FAKE_SHELLCHECK_VERSION
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
log=$work/log
bin=$work/bin
mkdir -p "$bin"

# Each fake logs "<tool> <args> @ <dir> a=<a.txt> b=<b.txt>" and fails when
# its arguments contain $FAIL.
for tool in cargo shellcheck python3; do
    cat >"$bin/$tool" <<EOF
#!/bin/sh
case "\$*" in --version) echo "version: \${FAKE_SHELLCHECK_VERSION:-0.11.0}"; exit 0 ;; esac
case "\$*" in *"import torch"*) [ -n "\${HAS_TORCH:-}" ]; exit \$? ;; esac
echo "$tool \$* @ \$PWD a=\$(cat a.txt 2>/dev/null) b=\$(cat b.txt 2>/dev/null)" >>"$log"
case "\$*" in *"\${FAIL:-<none>}"*) exit 1 ;; esac
EOF
done
chmod +x "$bin"/*

repo=$work/repo
mkdir -p "$repo/scripts/build" "$repo/cli/tests/scripts" "$repo/cli/tests/fixtures" "$repo/training"
cp "$ROOT/scripts/check.sh" "$repo/scripts/"
touch "$repo/install.sh" "$repo/scripts/build/x.sh" "$repo/cli/tests/fixtures/make-traces.sh"
printf '#!/bin/sh\necho "script test-one" >>"%s"\n' "$log" >"$repo/cli/tests/scripts/test-one.sh"
chmod +x "$repo/cli/tests/scripts/test-one.sh"
echo 'print(1)' >"$repo/training/test_plain.py"
printf 'import torch\n' >"$repo/training/test_torch.py"
echo committed >"$repo/a.txt"
echo committed >"$repo/b.txt"
git -C "$repo" init --quiet
git -C "$repo" add -A
git -C "$repo" commit --quiet -m init

# run [ARGS...]: runs the script with the fakes first on PATH.
run() {
    : >"$log"
    PATH=$bin:$PATH "$repo/scripts/check.sh" "$@" >"$work/out" 2>&1
}
check() {
    local name=$1
    shift
    if "$@"; then
        echo "ok    $name"
    else
        echo "FAIL  $name"
        sed 's/^/        /' "$work/out" "$log"
        failures=$((failures + 1))
    fi
}
logged() { grep -qF -- "$1" "$log"; }
said() { grep -qF -- "$1" "$work/out"; }

run && status=0 || status=$?
check "passes when every check passes" [ "$status" = 0 ]
check "runs cargo fmt, clippy and test" \
    sh -c "grep -q '^cargo fmt --check @ $repo ' '$log' &&
        grep -q '^cargo clippy --locked --workspace --all-targets -- -D warnings @' '$log' &&
        grep -q '^cargo test --locked --workspace @' '$log'"
check "runs shellcheck on CI's files" logged "shellcheck install.sh scripts/check.sh scripts/build/x.sh cli/tests/fixtures/make-traces.sh cli/tests/scripts/test-one.sh @"
check "runs every script test" logged "script test-one"
check "runs the Python tests isolated" logged "python3 -I training/test_plain.py @"
check "skips a torch test without torch, and says so" \
    sh -c "! grep -q test_torch '$log' && grep -q 'not run: training/test_torch.py: needs torch' '$work/out'"

HAS_TORCH=1 run
check "runs a torch test with torch" logged "python3 -I training/test_torch.py @"

FAIL="--workspace" run && status=0 || status=$?
check "fails when a check fails" [ "$status" = 1 ]
check "keeps going after a failure" logged "script test-one"
check "names the failed checks" said "FAIL     cargo test --locked --workspace"

SHELLCHECK=$work/none run && status=0 || status=$?
check "without shellcheck, passes and says it did not run" \
    sh -c "[ $status = 0 ] && grep -q 'not run: shellcheck: not installed' '$work/out'"

FAKE_SHELLCHECK_VERSION=0.9.0 run
check "warns when shellcheck is not CI's version" said "warning: shellcheck is 0.9.0, CI uses 0.11.0"

run --nope && status=0 || status=$?
check "refuses unknown arguments" [ "$status" = 2 ]

run --staged && status=0 || status=$?
check "--staged with nothing staged fails" sh -c "[ $status = 1 ] && grep -q 'nothing is staged' '$work/out'"

echo staged >"$repo/a.txt"
git -C "$repo" add a.txt
echo unstaged >"$repo/b.txt"
run --staged && status=0 || status=$?
check "--staged passes" [ "$status" = 0 ]
check "--staged runs outside the working tree" sh -c "! grep -q '^cargo fmt --check @ $repo ' '$log'"
check "--staged sees the staged change and not the unstaged one" logged "a=staged b=committed"
check "--staged removes its worktree" [ "$(git -C "$repo" worktree list | wc -l)" = 1 ]
check "--staged leaves the index and the working tree alone" \
    sh -c "[ \"\$(git -C '$repo' diff --staged --name-only)\" = a.txt ] && [ \"\$(cat '$repo/b.txt')\" = unstaged ]"

# The script and CI check the same shell files.
ci=$(sed -n 's/.*shellcheck --version && shellcheck \(.*\)$/\1/p' "$ROOT/.github/workflows/ci.yml")
check "lints the files CI lints" grep -qF "\"\$SHELLCHECK\" $ci" "$ROOT/scripts/check.sh"
check "pins the shellcheck version CI uses" \
    grep -q "^SHELLCHECK_VERSION=$(sed -n 's/.*SHELLCHECK_VERSION: "\(.*\)"/\1/p' "$ROOT/.github/workflows/ci.yml") " "$ROOT/scripts/check.sh"

[ "$failures" = 0 ] || {
    echo "$failures check(s) failed" >&2
    exit 1
}
echo "all checks passed"
