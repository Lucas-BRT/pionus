<div align="center">

<img src="website/assets/logo.svg" alt="" width="128">

# pionus

Commit messages in your repository's style, from a small language model that
runs on your machine.

[![CI](https://github.com/Lucas-BRT/pionus/actions/workflows/ci.yml/badge.svg)](https://github.com/Lucas-BRT/pionus/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/tag/Lucas-BRT/pionus?filter=v*&label=release&sort=semver)](https://github.com/Lucas-BRT/pionus/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 1.99](https://img.shields.io/badge/rust-1.99-orange?logo=rust)](rust-toolchain.toml)
[![Linux x86_64](https://img.shields.io/badge/platform-linux%20x86__64-lightgrey?logo=linux&logoColor=white)](docs/usage.md#install)

</div>

pionus reads your staged diff and your history and writes the commit message.
The model runs on an Intel NPU, an Intel, AMD or NVIDIA GPU, or the CPU. No
account, no API key, and nothing leaves your computer.

```console
$ git add -p
$ pionus
feat(diff): share the token budget between files by priority
```

## Features

- **Matches your history.** Learns the convention (Conventional Commits,
  gitmoji or free form), the scopes and the ticket format.
- **Runs on what you have.** Intel NPUs and GPUs through OpenVINO, AMD and
  NVIDIA GPUs through llama.cpp and Vulkan, or any x86_64 CPU. The setup
  installs only what your hardware needs and picks the fastest device.
- **Works with any git client.** A `prepare-commit-msg` hook fills in the
  message of every `git commit` and never blocks one.
- **Reproducible.** The toolchain, dependencies, drivers, runtimes and models
  are pinned by version and hash, the release tarballs are byte-identical on
  any machine, and any generation can be recorded and replayed.

Writing a message is local. Installing, updating and downloading models use
the network; the daily check for a newer release is off until you turn it on
([update notice](docs/usage.md#update-notice)).

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/Lucas-BRT/pionus/main/install.sh | bash
```

This installs pionus in `~/.local` without `sudo` and runs the guided setup: it
lists your hardware, asks before installing drivers, downloads the model and
benchmarks the devices. To read the plan first, use `--no-setup`, then
`pionus setup --dry-run`.

Linux on x86_64 only. Drivers are installed with `apt` (Ubuntu 24.04 and
derivatives are tested); on other distributions the setup lists the packages.
Installer options and building from source are in the
[usage guide](docs/usage.md#install).

`pionus update` installs a newer release and keeps your models and settings.
`pionus uninstall` lists and removes everything pionus downloaded.

## Usage

| Command | Does |
|---|---|
| `pionus` | print a message for what is staged |
| `pionus --commit` | review the message (commit, edit, ask for another), then commit |
| `pionus hook install` | fill in the message of every `git commit` in this repository |
| `pionus model` / `model use <id>` | show or change the [model](docs/models.md) |
| `pionus device` / `device use <device>` | show or change the device |
| `pionus bench` | measure this machine's devices |
| `pionus doctor` | check the installation and say how to fix what is wrong |
| `pionus config list --show-origin` | show every setting and where it comes from |

The other commands (`setup`, `hardware`, `context`, `prompt`, `server`,
`--trace` and `replay`) are in the [usage guide](docs/usage.md).

## Supported hardware

| Device | Backend | Hardware |
|---|---|---|
| `npu` | OpenVINO GenAI | Intel NPU (Core Ultra) |
| `intel-gpu` | OpenVINO GenAI | Intel GPU, integrated or Arc |
| `vulkan` | llama.cpp | AMD (RADV), NVIDIA (proprietary or NVK) and Intel GPUs |
| `cpu` | llama.cpp | any x86_64 CPU, Intel or AMD |
| `openvino-cpu` | OpenVINO GenAI | any x86_64 CPU, where OpenVINO is installed |

The default model is Qwen2.5-Coder-1.5B-Instruct. Times per message are in
[measurements](docs/measurements.md); how the device is chosen is in
[devices](docs/devices.md).

## Documentation

- [Usage](docs/usage.md): install, everyday use, hook, troubleshooting
- [Configuration](docs/configuration.md): every setting and its layers
- [Devices](docs/devices.md): backends, device selection, hardware setup
- [Measurements](docs/measurements.md): how long a message takes
- [Models](docs/models.md): the recommended models, and using your own
- [Architecture](docs/architecture.md): how a message is made
- [Reproducibility](docs/reproducibility.md): what is pinned and how it is checked
- [Training](docs/training.md): how pionus's own models are trained
- [Roadmap](docs/roadmap.md): what comes next
- [Contributing](docs/contributing.md): checks, tests, commits and releases

## Contributing

`scripts/check.sh` runs everything CI runs, and the tests need no hardware.
The rules for changes are in [AGENTS.md](AGENTS.md).

## License

[MIT](LICENSE). The models are not part of it: they are downloaded under their
own licences (the recommended ones are Apache-2.0).
