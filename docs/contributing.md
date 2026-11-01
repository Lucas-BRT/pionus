# Contributing

How changes are checked, tested, committed and released.
[AGENTS.md](../AGENTS.md) has the rules; this page has the details.

## Checks

```bash
scripts/check.sh            # everything CI runs, plus the Python tests
scripts/check.sh --staged   # the same, on the staged change alone
```

`--staged` applies the staged diff to a throwaway worktree, so a commit is
checked without what is left unstaged. The script lists what failed and what
it could not run: shellcheck missing or not 0.11.0 (the version CI pins),
the Python tests that need torch. `cli/tests/scripts/test-install.sh` needs
Docker. Website changes also need the visual check of
[website/AGENTS.md](../website/AGENTS.md).

## Tests

- **New behaviour:** the normal case and the edges (empty input, limits,
  errors).
- **Bug fix:** a regression test that fails without the fix; check it by
  restoring the old code.
- **Refactoring:** the existing tests pass unchanged; if one must change, the
  commit message says why.

| Kind | Where |
|---|---|
| Unit | under `cli/src/`: `tests.rs` beside `mod.rs` for a module with its own directory, else `#[cfg(test)] mod tests` |
| Prompt snapshots | `insta::assert_snapshot!`, files in `snapshots/` beside the module; update with `INSTA_UPDATE=always cargo test` and review the diff |
| Git integration | `cli/tests/git_context.rs`, with `TestRepo` from `cli/tests/common/` |
| CLI end to end | `cli/tests/{cli,config_cli,bench_cli,hook,setup_cli}.rs`, running `CARGO_BIN_EXE_smart-commit` |
| Trace fixtures | `cli/tests/replay_fixtures.rs` |
| Docs | `cli/tests/docs.rs`: the paths the pages name exist, every command is in `docs/usage.md`, links resolve |
| Tools | inside `tools/` and `training/tools/`; the scenarios built twice need `cargo test -- --ignored` after `cargo build` |
| Scripts | `cli/tests/scripts/test-*.sh`, against fake sysfs trees, releases and binaries; no network, sudo or hardware; each one is a step of `.github/workflows/ci.yml` |
| Python | `training/test_*.py`, run with `python3 -I` |
| Website | `website/tests/routes.rs`: every page, and the facts it takes from the product |
| Hardware | `cli/tests/hardware.rs`, ignored by default: `cargo test --test hardware -- --ignored --test-threads 1` with `SMART_COMMIT_TEST_DEVICE=npu\|intel-gpu\|vulkan\|cpu\|openvino-cpu`; fixture prompts only, never ad hoc inputs |

`cargo test` depends on no NPU or GPU, network, clock or randomness, and on
none of the machine's git config, smart-commit config, models or
`SMART_COMMIT_*` variables: run commands through `cli/tests/common::isolate`.
Nothing that reaches the output depends on `HashMap` iteration order.

## Commits

- **One purpose per commit.** Split a file with changes for two purposes by
  hunk: `git add -p FILE`, or without a terminal, `git apply --cached` a
  partial patch. Look at `git diff --staged` and `git diff`, then run
  `scripts/check.sh --staged`.
- **Commit by explicit path** when the index holds files that are not yours
  (a teammate's, a local `mise.toml`); never unstage or include them:
  `git commit -F message.txt -- path/one path/two`.
- **Messages:** Conventional Commits in English (`feat: ...`,
  `fix(diff): ...`, `docs: ...`, `test: ...`), subject of at most 72
  columns; a breaking change adds `!` and a `Breaking:` paragraph. The body
  says why when that is not obvious. Release notes are made from subjects.
- **Ask smart-commit first.** With only the commit's changes staged:

  ```bash
  smart-commit --trace /tmp/trace.json                      # the installed release
  target/release/smart-commit --trace /tmp/trace-dev.json   # this tree
  ```

  Compare its message with yours. A wrong one (type, scope, invented details,
  a copied history subject, a fallback) is a bug found in real use: make it a
  fixture in `cli/tests/fixtures/traces/` with
  `cli/tests/fixtures/make-traces.sh`, in a commit of its own. Without the
  trace it cannot be investigated.
- **Dependency updates** get commits of their own.

## Releases

Bump `version` in `cli/Cargo.toml` (semver) in a commit
`chore: release X.Y.Z` on `main`. `.github/workflows/release.yml` tests,
packages and publishes the release, with notes from the commit subjects
(`sc-tools release-notes`). Tags and releases are never made by hand. The
tarball is reproducible: `cli/tests/scripts/test-install.sh` packages twice
and requires identical bytes, and CI prints the hash of every commit
([reproducibility](reproducibility.md#builds-and-releases)).
