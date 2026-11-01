# AGENTS.md

Rules for anyone changing this repo. A directory's `AGENTS.md` adds to this
file, never overrides it.

pionus: Rust CLI that writes commit messages in a repo's style with a
small local language model (Intel NPU, Intel GPU via OpenVINO, AMD/NVIDIA via
llama.cpp + Vulkan, or CPU). Virtual Cargo workspace: `cli/` (the binary, only
default member), `website/` (read [website/AGENTS.md](website/AGENTS.md)),
`tools/` (`pionus-tools`), `training/tools/` (`pionus-training`; the rest of
`training/` is Python). Shared deps go in the root `Cargo.toml`.

## Hard rules

| Rule | Caught by |
|---|---|
| Tests ship in the same commit; a bug fix's test fails without the fix | review |
| One purpose per commit; each passes `scripts/check.sh --staged` | review, CI |
| Subjects: Conventional Commits, English, ≤ 72 columns | review, release notes |
| No tags or releases by hand: bump `cli/Cargo.toml` in `chore: release X.Y.Z` | release workflow |
| Fixtures and traces are generated (`cli/tests/fixtures/make-traces.sh`), never hand-edited | `replay_fixtures` |
| Numbers and commands in docs and site come from the repo, never memory | `docs.rs`, website tests |
| Everything reaching generation is pinned: versions, SHA-256, explicit decoding params | CI (`--locked`, hashes) |
| Index holds files that are not yours: `git commit -F msg -- <your paths>` | review |

## Workflow

- **Checks:** `scripts/check.sh` before any commit; it says what it skipped.
- **Before each commit**, run `pionus --trace /tmp/trace.json` on the
  staged change. A wrong message becomes a trace fixture, in its own commit.
- **Read first:** tests, commits and releases: [docs/contributing.md](docs/contributing.md).
  Pinning, export, traces, build image: [docs/reproducibility.md](docs/reproducibility.md).
  Models and Hugging Face: [docs/models.md](docs/models.md).
- `cargo test` never depends on hardware, network, clock, randomness or the
  machine's config: run commands through `cli/tests/common::isolate`. Inputs
  live in `cli/tests/fixtures/` or inline, never at local paths. Never let
  `HashMap` order reach output.
- **Generation:** greedy, every parameter explicit (`GenerationConfig`), never
  library defaults; a retry changes the prompt deterministically. An intended
  pipeline change regenerates the traces in the same commit.
- **Config:** every setting in `config::KEYS`, never read elsewhere; machine
  keys are `Scope::User`; settings that change output reach traces via
  `PromptOptions` or `GenerationConfig`; list it in `docs/configuration.md`.
- **Docs:** describe the current state, in English; update the affected pages
  and `docs/roadmap.md` in the same commit. Timings live only in
  `docs/measurements.md`. The site reads its facts from `cli/Cargo.toml`,
  `cli/src/models/registry.rs` and `docs/{devices,usage,roadmap}.md`: change
  those, not the site.

## Done

Checks run (not assumed) on what you committed; docs and roadmap follow the
change; the report says what you did not check or run.

## Changing this file

Only what code and docs do not tell. Same correction twice: add a rule with
what catches it. Enforced by CI or a test: one line. Irrelevant for long:
delete. Directory-only rules go in that directory's `AGENTS.md`.
