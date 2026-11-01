<div align="center">

<img src="website/assets/logo.svg" alt="" width="128">

# pionus

Commit messages in your repository's style, from a small language model that
runs on your machine.

[![CI](https://github.com/Lucas-BRT/pionus/actions/workflows/ci.yml/badge.svg)](https://github.com/Lucas-BRT/pionus/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/tag/Lucas-BRT/pionus?filter=v*&label=release&sort=semver)](https://github.com/Lucas-BRT/pionus/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Linux x86_64](https://img.shields.io/badge/platform-linux%20x86__64-lightgrey?logo=linux&logoColor=white)](docs/usage.md#install)

</div>

pionus reads your staged diff and your history and writes the commit message.
The model runs on an Intel NPU, a GPU or the CPU. No account, no API key, and
writing a message never leaves your computer ([network use](docs/usage.md#network-use)).

```console
$ git add -p
$ pionus
feat(diff): share the token budget between files by priority
```

## Features

- **Matches your history.** Learns the convention, the scopes and the ticket
  format.
- **Runs on what you have.** Intel NPU and GPU, AMD and NVIDIA GPU, or any
  x86_64 CPU; the setup picks the fastest ([devices](docs/devices.md)).
- **Works with any git client.** A `prepare-commit-msg` hook fills in every
  `git commit` and never blocks one.
- **Reproducible.** Pinned by version and hash; the release tarballs are
  byte-identical on any machine ([how](docs/reproducibility.md)).

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/Lucas-BRT/pionus/main/install.sh | bash
```

For Linux on x86_64, without `sudo`. A guided setup then lists your hardware,
asks before installing drivers, downloads the model and benchmarks the
devices. Options, update and uninstall: [usage guide](docs/usage.md#install).

## Usage

```bash
pionus               # print a message for what is staged
pionus --commit      # review it, then commit
pionus hook install  # fill in every `git commit` of this repository
pionus doctor        # check the installation
```

More in the [usage guide](docs/usage.md#everyday-use).

## Documentation

Every page is in [docs/](docs/): [usage](docs/usage.md),
[configuration](docs/configuration.md), [models](docs/models.md),
[measurements](docs/measurements.md) and
[contributing](docs/contributing.md).

## License

[MIT](LICENSE). The models are downloaded under their own licences.
