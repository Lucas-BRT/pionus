# Configuration

pionus works without any configuration. Settings exist to force a
style for a team, to pick a device, or to point at models in other places.

```console
$ pionus config init                          # ~/.config/pionus/config.toml, every key commented out
$ pionus config set model.device cpu          # global
$ pionus config set --repo style.convention conventional   # .pionus.toml, shared through git
$ pionus config get style.convention
$ pionus config list --show-origin
backend = auto  (default)
model.device = cpu  (global /home/me/.config/pionus/config.toml)
style.convention = conventional  (repository /home/me/project/.pionus.toml)
...
$ pionus config unset model.device
$ pionus config path [--repo]
```

## Layers

From strongest to weakest:

1. command-line flags (`--backend`, `--device`, `--model-dir`);
2. environment variables (listed below);
3. the repository's `.pionus.toml`, at the root of the work tree;
4. the global `$XDG_CONFIG_HOME/pionus/config.toml` (by default
   `~/.config/pionus/config.toml`);
5. the built-in defaults.

`config list --show-origin` shows which layer each value comes from.
`config set` and `config unset` warn when a stronger layer still decides the
value, for example when an environment variable is set.

## Keys

| Key | Values | Default | Environment | Where |
|---|---|---|---|---|
| `backend` | `auto`, `openvino`, `llamacpp`, `mock` | `auto` | `PIONUS_BACKEND` | global |
| `model.device` | `auto`, `npu`, `intel-gpu`, `vulkan`, `cpu`, `openvino-cpu` | `auto` | `PIONUS_DEVICE` | global |
| `model.name` | `qwen2.5-coder-1.5b`, `qwen2.5-coder-0.5b`, `qwen2.5-coder-7b`, `pionus-1.5b-v0.1`, `pionus-0.5b-v0.1`, `pionus-0.5b-v0.2`, `pionus-1.5b-v0.2`, `pionus-0.5b-v0.3`, `pionus-1.5b-v0.3` | `qwen2.5-coder-1.5b` | `PIONUS_MODEL` | global |
| `model.dir` | path | model.name's OpenVINO export in `$XDG_DATA_HOME/pionus/models/` (none for models without one) | `PIONUS_MODEL_DIR` | global |
| `model.cache_dir` | path | `$XDG_CACHE_HOME/pionus/openvino` | `PIONUS_CACHE_DIR` | global |
| `model.gguf` | path | model.name's GGUF file in `$XDG_DATA_HOME/pionus/models/` | `PIONUS_GGUF` | global |
| `model.llama_server` | path | `$XDG_DATA_HOME/pionus/llama.cpp/llama-b11381/llama-server` | `PIONUS_LLAMA_SERVER` | global |
| `model.vulkan_device` | 0–15 | 0 | | global |
| `model.fast_load` | `auto`, `on`, `off` | `auto` | `PIONUS_FAST_LOAD` | global |
| `model.threads_batch` | 0–1024 | 0 (one per physical core) | | global |
| `model.keep_loaded` | 0–1440 | 10 | `PIONUS_KEEP_LOADED` | global |
| `model.keep_loaded_max_ram` | 1–100 | 15 | `PIONUS_KEEP_LOADED_MAX_RAM` | global |
| `model.ask_type` | `off`, `low_confidence` | `off` | `PIONUS_ASK_TYPE` | global |
| `model.ask_type_below` | 1–99 | 50 | `PIONUS_ASK_TYPE_BELOW` | global |
| `model.npu_prefill_chunk` | 512–8192 | 512 | | global |
| `model.npu_generate_hint` | `best_perf`, `fast_compile` | `best_perf` | | global |
| `model.max_prompt_tokens` | 256–8192 | 1024 | | global |
| `model.max_new_tokens` | 16–1024 | 128 | | global |
| `style.convention` | `auto`, `conventional`, `gitmoji`, `free` | `auto` | | global or repository |
| `style.default_convention` | `conventional`, `gitmoji`, `free` | `conventional` | | global or repository |
| `style.history_depth` | 1–500 | 30 | | global or repository |
| `style.subject_max` | 40–200 | 72 | | global or repository |
| `update.check` | `false`, `true` | `false` | `PIONUS_UPDATE_CHECK` | global |

- `backend = auto` uses whichever backend the device needs: OpenVINO for
  `npu`, `intel-gpu` and `openvino-cpu`, llama.cpp for `vulkan` and `cpu`.
  `mock` replies with `--mock-response` / `PIONUS_MOCK_RESPONSE`, for
  tests.
- `model.device = auto` uses the fastest device of the last
  `pionus bench`, and the first that works of intel-gpu, npu, vulkan,
  cpu and openvino-cpu when there is no benchmark. See [devices](devices.md).
  `gpu`, the name of `intel-gpu` before 0.4, still works with a warning;
  `config set` and `device use` write the new name.
- `model.name` is one of the [recommended models](models.md);
  `pionus model use <id>` downloads it and sets it. `model.gguf` and
  `model.dir` default to its files, whichever layer sets it; set them (or
  `pionus model use FILE`) to run a model from disk instead.
- `model.vulkan_device` picks a GPU on machines with more than one, as
  `llama-server --list-devices` numbers them (0 is `Vulkan0`).
- `model.fast_load` starts llama-server (`vulkan` and `cpu`) with
  `--no-repack --fit off`: the weights are used as they are mapped from the
  file instead of being copied and repacked for the CPU's vector
  instructions, and the model is not fitted to the GPUs' free memory, which
  a 1 GB model with a fixed context rarely needs. `auto` uses it on the
  devices where the last `pionus bench` found it at least 5% faster;
  without a benchmark it stays off. If a fast load fails (a GPU short of
  memory), the model is loaded again normally, with a note. See
  [devices](devices.md#fast-load).
- `model.threads_batch` sets llama-server's `--threads-batch`, the
  threads that read the prompt on the `cpu` device, which is most of the
  time per message there. 0 means one per physical core, from the CPU
  topology in sysfs. llama.cpp's own default already does that on some
  processors and not on others; see
  [measurements](measurements.md#threads-on-the-cpu). Decoding keeps
  llama.cpp's default thread count.
- `model.keep_loaded` keeps the model loaded on `vulkan` and `cpu` for that
  many minutes after a message (10 by default; 0 stops it after each command), so the next commit skips loading it, which
  is most of a commit on a GPU. It costs the model's memory (RAM or VRAM)
  while it stays loaded, so it is limited by `model.keep_loaded_max_ram`
  (below). The server listens in `$XDG_RUNTIME_DIR`; without
  that variable, or if it cannot start, the model is loaded for the commit
  only, with a note. Replies are the same. `pionus server status`
  lists what is kept loaded, `pionus server stop` stops it. Changing a
  `model.*` setting with pionus (`config set` or `unset`, `model use`,
  `device use`) stops it too, since it runs the old settings; a load that
  fails while a model is kept says so.
- `model.keep_loaded_max_ram` is the share of the machine's RAM, in percent
  (15 by default), a model kept loaded may take. pionus estimates what
  the server takes (the model file and about 256 MB, see
  [measurements](measurements.md#keeping-the-model-loaded)) and does not keep
  a model that would take more of the total RAM than that, or whose loading
  would leave less than a tenth of the RAM available: it is loaded for the
  commit only, which is said only if `model.keep_loaded` was set by someone
  (a model that is too large for the machine is the default's business, not
  an error). A kept server is checked every few seconds and stops, however
  idle or busy, when its resident memory passes the share or when less than a
  tenth of the RAM is available. On 16 GiB the default keeps the 0.5B and
  the 1.5B models (4% and 7%) and not the 7B (30%); on a machine of 4 GiB it
  keeps none (16% and 31%), and 20 would keep the 0.5B. 100 removes the share,
  not the tenth that stays available. Without a readable `/proc/meminfo` nothing stops a
  model from being kept.
- `update.check` set to `true` lets the commands a person runs by hand ask
  GitHub whether a newer release is out, at most once in 24 hours, and print
  a line on stderr if so. It is off by default; see the
  [update notice](usage.md#update-notice) for what is sent and when.
  In a file it is written `check = true` or `check = "true"`.
- `model.ask_type` set to `low_confidence` makes `pionus --commit` in a
  terminal ask which type the message should have when the model is unsure:
  the model's first token is where it picks the type, and its probability
  tells how sure it is. Below `model.ask_type_below` percent (50 by default)
  it offers its likeliest types (at most four, only the ones the repository
  uses, as the prompt lists them) and `other`, which leaves the choice to the
  model; the message is then written for the type chosen. It is only on the
  `vulkan` and `cpu` devices (llama.cpp), only for Conventional Commits and
  only with a terminal: the `prepare-commit-msg` hook, a pipe and the OpenVINO
  devices keep the model's own choice. The offer and the choice are in the
  trace. What it gains is in
  [measurements](measurements.md#asking-for-the-type).
- `model.npu_prefill_chunk` must divide `model.max_prompt_tokens`. Chunks
  smaller than 512 fail to compile with NPU driver 1.38.
- `model.max_prompt_tokens` is also the prompt length the NPU compiles for;
  the NPU is given a fifth less of it as the prompt budget, since it fails
  on a longer prompt and the estimate can be short
  ([architecture](architecture.md#fitting-the-diff)).
  Shorter prompts are faster on the NPU but see less of the diff; changing it
  recompiles the model once (see [measurements](measurements.md#the-npu)).
- `style.convention` other than `auto` overrides what the history says. With
  a forced convention, history examples that contradict it are left out of
  the prompt.
- `style.default_convention` applies when `auto` has nothing to go on: no
  history, or one that follows no convention.
- Messages are written in English. `style.language` and
  `style.default_language`, which asked for Portuguese, were removed. A
  configuration file that still has them keeps working (they are read and
  ignored), but they can no longer be set, and the lines can be deleted.
- `style.subject_max` is the longest subject, in columns. 72 follows git's
  convention (GitHub cuts the subject about there, `git log --oneline` and
  patches by e-mail assume short lines); a team whose commitlint allows 100
  sets it for the repository: `pionus config set --repo
  style.subject_max 100`. The prompt states the limit, also to a model
  trained without the fixed rules when it is not 72, and a longer reply ends
  at the last phrase that fits (see [architecture](architecture.md)).

Two more environment variables have no key: `PIONUS_MOCK_RESPONSE`
(a reply of the mock backend) and `PIONUS_TRACE` (record every
generation to that file, see [reproducibility](reproducibility.md)).

A few more exist for tests and scripts, not for configuring a machine:
`PIONUS_SYSFS`, `PIONUS_CPUINFO` and `PIONUS_OPENVINO_DIR`
point the hardware inventory at a fake tree (`scripts/setup-gpu.sh` reads
`PIONUS_SYSFS` too), `PIONUS_HF_BASE` replaces the Hugging Face
URL models are downloaded from, and `PIONUS_LLAMACPP_URL` the one
llama.cpp is (the pinned SHA-256 is still required).

## Rules

- **Unknown keys and invalid values are errors**, with the file and the key
  in the message: a mistyped `model.devise` never goes unnoticed. In the git
  hook the error becomes a warning, so a broken file never blocks a commit.
  `pionus setup` removes from the global file the values this version
  cannot read, such as a model added by a newer build, with a warning that
  names the old value, so the defaults apply and the setup still runs.
- **Machine keys are global only.** `backend` and every `model.*` key are
  refused in `.pionus.toml`, whether by `config set --repo` or by
  editing the file. Cloning a repository must not change which model runs
  on your machine, nor make it load a model from a path someone else chose.
- **Paths** are absolute or start with `~/`, so there is no question of what
  a relative one is relative to. A relative `--model-dir` on the command line
  is resolved from the current directory, as expected from a flag.
- **`config set` keeps comments and layout.** In a file written by
  `config init`, setting a key replaces its commented line, right under its
  description.
- Settings that change what is generated (`style.*`, `model.max_prompt_tokens`,
  `model.max_new_tokens`) are recorded in traces, so `pionus replay`
  does not depend on the config files of whoever runs it.

Every key is declared once, in `config::KEYS`, which drives reading,
validation, `config set`, the listing and the `config init` template alike.
A test checks that this page lists every key.
