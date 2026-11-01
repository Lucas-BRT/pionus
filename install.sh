#!/usr/bin/env bash
# Installs pionus from its GitHub releases, without sudo, then runs the
# guided setup (drivers, models and the benchmark that picks the device).
#
#   curl -fsSL https://raw.githubusercontent.com/Lucas-BRT/pionus/main/install.sh | bash
#
# Options (after `bash -s --` when piping):
#   --version X.Y.Z  install that release instead of the latest
#   --prefix DIR     install in DIR/bin and DIR/share (default: ~/.local)
#   --no-setup       only install; run `pionus setup` later
#   --update         install the release only if it is not the version
#                    installed, without the setup; llama.cpp, if installed,
#                    follows the build the new release pins (no sudo).
#                    `pionus update` runs this
#   --verify         also verify the build provenance (a GitHub attestation)
#                    with `gh attestation verify`; needs the GitHub CLI, and
#                    releases up to 0.14.0 have none
#   --yes            answer yes to the setup's questions
#   --uninstall      remove pionus (models, caches and config are kept)
#   --purge          --uninstall, and also the models, llama.cpp, the
#                    benchmark, the caches and the configuration
#                    (`pionus uninstall` runs this)
#   --system         --uninstall, and also OpenVINO, if pionus's setup
#                    installed it (/opt/intel, /etc/ld.so.conf.d; uses sudo,
#                    asks first unless --yes). Driver packages are kept.
#
# The download is checked against the release's SHA256SUMS.
# PIONUS_RELEASE_BASE overrides where releases are downloaded from;
# PIONUS_SYSTEM_ROOT (a directory standing for /) and
# PIONUS_SUDO (the command instead of sudo), PIONUS_OS_RELEASE
# (the os-release file the git hint reads) and PIONUS_GETCONF (the
# command that reports the C library) are for tests.
set -euo pipefail

BASE=${PIONUS_RELEASE_BASE:-https://github.com/Lucas-BRT/pionus/releases}
ASSET=pionus-x86_64-unknown-linux-gnu.tar.gz
PREFIX=$HOME/.local
VERSION=latest
SETUP=1
UPDATE=0
UNINSTALL=0
PURGE=0
SYSTEM=0
ASSUME_YES=0
VERIFY=0
setup_args=()

# The oldest glibc the release runs on: the one of the Ubuntu 22.04 container
# that builds it (scripts/build/Dockerfile).
MIN_GLIBC=2.35
# Free space the setup wants, in KiB: the default model (1.1 GB), llama.cpp
# unpacked (about 0.1 GB) and a margin.
MIN_FREE_KB=1500000

while [ $# -gt 0 ]; do
    case $1 in
        --version)
            VERSION=${2:?--version needs a value}
            VERSION=${VERSION#v}
            shift
            ;;
        --prefix)
            PREFIX=${2:?--prefix needs a directory}
            shift
            ;;
        --no-setup) SETUP=0 ;;
        --verify) VERIFY=1 ;;
        --update) UPDATE=1 SETUP=0 ;;
        --yes)
            setup_args+=(--yes)
            ASSUME_YES=1
            ;;
        --uninstall) UNINSTALL=1 ;;
        --purge) UNINSTALL=1 PURGE=1 ;;
        --system) UNINSTALL=1 SYSTEM=1 ;;
        -h | --help)
            sed -n '2,/^set -euo/p' "$0" | sed '$d; s/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "unknown option: $1" >&2
            exit 2
            ;;
    esac
    shift
done

say() { printf '==> %s\n' "$*"; }

# What the release puts in share/pionus. With the default prefix that
# directory is also $XDG_DATA_HOME/pionus, where the setup keeps the
# models, llama.cpp and the benchmark: only these entries are ever replaced
# or removed.
RELEASE_FILES=(scripts VERSION)

# remove_release: the files this script installed, and nothing else.
remove_release() {
    local entry
    rm -f "$PREFIX/bin/pionus"
    for entry in "${RELEASE_FILES[@]}"; do
        rm -rf "${PREFIX:?}/share/pionus/$entry"
    done
    rmdir "$PREFIX/share/pionus" 2>/dev/null || true
    rm -rf "${PREFIX:?}/share/doc/pionus"
}
die() {
    echo "error: $*" >&2
    exit 1
}

# ask QUESTION: yes with --yes; no without a terminal to ask on.
ask() {
    [ "$ASSUME_YES" = 1 ] && return 0
    if ! { : </dev/tty; } 2>/dev/null; then
        echo "$1 [skipped: no terminal; pass --yes]"
        return 1
    fi
    printf '%s [y/N] ' "$1"
    local answer
    read -r answer </dev/tty || return 1
    case $answer in y | Y | yes | s | S | sim) return 0 ;; *) return 1 ;; esac
}

# remove_openvino: undoes setup-npu.sh's OpenVINO install. Its ld.so.conf.d
# file marks an install of ours; without it nothing is touched, so an
# OpenVINO installed some other way stays.
remove_openvino() {
    local root=${PIONUS_SYSTEM_ROOT:-}
    local sudo=${PIONUS_SUDO-sudo}
    local conf=$root/etc/ld.so.conf.d/pionus-openvino.conf
    local link=$root/opt/intel/openvino target=
    if [ ! -e "$conf" ]; then
        say "no OpenVINO installed by pionus; nothing to remove"
        return
    fi
    if [ -L "$link" ]; then
        target=$(readlink "$link")
        case $target in
            /opt/intel/openvino_* | "$root"/opt/intel/openvino_*) target=$root/opt/intel/${target##*/} ;;
            *) target= ;;
        esac
    fi
    say "OpenVINO installed by pionus's setup:"
    echo "  $conf"
    [ -L "$link" ] && echo "  $link"
    [ -n "$target" ] && echo "  $target"
    ask "Remove it (uses sudo)?" || return 0
    $sudo rm -f "$conf"
    [ -L "$link" ] && $sudo rm -f "$link"
    [ -n "$target" ] && $sudo rm -rf "$target"
    $sudo rmdir "$root/opt/intel" 2>/dev/null || true
    # Only the real system's linker cache knows these libraries.
    [ -n "$root" ] || $sudo ldconfig
    say "removed OpenVINO"
}

if [ "$UNINSTALL" = 1 ]; then
    remove_release
    say "removed pionus from $PREFIX"
    data=${XDG_DATA_HOME:-$HOME/.local/share}/pionus
    cache=${XDG_CACHE_HOME:-$HOME/.cache}/pionus
    config=${XDG_CONFIG_HOME:-$HOME/.config}/pionus
    if [ "$PURGE" = 1 ]; then
        rm -rf "${data:?}" "${cache:?}" "${config:?}"
        say "removed the models, llama.cpp, the benchmark, the caches and the configuration"
    else
        echo "Kept (--purge removes them):"
        echo "  models and benchmark: $data"
        echo "  compiled-model cache: $cache"
        echo "  configuration:        $config"
    fi
    if [ "$SYSTEM" = 1 ]; then
        remove_openvino
    fi
    echo "Kept: driver packages (Intel NPU driver, Level Zero, Intel's compute runtime,"
    echo "Vulkan drivers) and render group membership; other programs may use them."
    echo "Repositories with the hook keep it; \`pionus hook uninstall\` needs the binary,"
    echo "so remove .git/hooks/prepare-commit-msg by hand there."
    exit 0
fi

[ "$(uname -s)" = Linux ] || die "pionus runs on Linux only"
[ "$(uname -m)" = x86_64 ] || die "pionus releases are built for x86_64 only"
for tool in curl tar gzip sha256sum; do
    command -v "$tool" >/dev/null || die "$tool is required"
done

# preflight: what the release and the setup need, said before downloading.
preflight() {
    local id='' id_like='' hint='install git' line libc found free dir
    local osr=${PIONUS_OS_RELEASE:-/etc/os-release}
    if ! command -v git >/dev/null; then
        if [ "$UPDATE" = 1 ]; then
            echo "warning: git is not installed; pionus needs it to read repositories." >&2
        else
            if [ -r "$osr" ]; then
                line=$(grep '^ID=' "$osr" || true)
                id=${line#ID=}
                line=$(grep '^ID_LIKE=' "$osr" || true)
                id_like=${line#ID_LIKE=}
            fi
            id=${id//\"/}
            id_like=${id_like//\"/}
            case " $id $id_like " in
                *" debian "* | *" ubuntu "*) hint='run: sudo apt install git' ;;
                *" fedora "* | *" rhel "* | *" centos "*) hint='run: sudo dnf install git' ;;
                *" arch "*) hint='run: sudo pacman -S git' ;;
                *suse*) hint='run: sudo zypper install git' ;;
            esac
            die "git is required (pionus reads your repository with it); $hint"
        fi
    fi

    libc=$(${PIONUS_GETCONF:-getconf} GNU_LIBC_VERSION 2>/dev/null || true)
    case $libc in
        "glibc "*)
            found=${libc#glibc }
            if [ "$(printf '%s\n%s\n' "$MIN_GLIBC" "$found" | sort -V | head -n 1)" != "$MIN_GLIBC" ]; then
                die "pionus's release needs glibc $MIN_GLIBC or newer (Ubuntu 22.04+, Debian 12+, Fedora 36+...); this system has glibc $found."
            fi
            ;;
        *)
            die "pionus's release needs glibc $MIN_GLIBC or newer (Ubuntu 22.04+, Debian 12+, Fedora 36+...); this system does not use glibc (musl, Alpine?)."
            ;;
    esac

    if [ "$SETUP" = 1 ]; then
        dir=${XDG_DATA_HOME:-$HOME/.local/share}
        while [ ! -d "$dir" ] && [ "$dir" != / ] && [ "$dir" != . ]; do dir=$(dirname "$dir"); done
        free=$(df -Pk "$dir" 2>/dev/null | awk 'NR==2 {print $4}')
        if [ -n "$free" ] && [ "$free" -lt "$MIN_FREE_KB" ]; then
            echo "warning: only $((free / 1024)) MB free in $dir; the default model needs about 1.5 GB." >&2
            echo "         Free some space first, or the setup may fail part way." >&2
        fi
    fi
}
preflight

if [ "$VERSION" = latest ]; then
    url=$BASE/latest/download
else
    url=$BASE/download/v$VERSION
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
say "downloading $url/$ASSET"
curl -fL --retry 3 -o "$work/$ASSET" "$url/$ASSET"
curl -fsL --retry 3 -o "$work/SHA256SUMS" "$url/SHA256SUMS"
(cd "$work" && grep " $ASSET\$" SHA256SUMS | sha256sum --check --quiet -) ||
    die "the download does not match the release's SHA256SUMS"
if [ "$VERIFY" = 1 ]; then
    command -v gh >/dev/null || die "--verify needs the GitHub CLI: https://cli.github.com"
    say "verifying the build provenance"
    if ! attestation=$(gh attestation verify "$work/$ASSET" --repo Lucas-BRT/pionus 2>&1); then
        printf '%s\n' "$attestation" >&2
        case $attestation in
            *[Nn]o\ attestation*) die "no attestation: releases up to 0.14.0 have none" ;;
            *) die "the build provenance of the download could not be verified" ;;
        esac
    fi
fi

tar -xzf "$work/$ASSET" -C "$work"
src=$work/pionus-x86_64-unknown-linux-gnu
new=$(cat "$src/share/pionus/VERSION")
old=$(cat "$PREFIX/share/pionus/VERSION" 2>/dev/null || true)
if [ "$UPDATE" = 1 ] && [ "$new" = "$old" ]; then
    say "pionus $old is already the latest release"
    exit 0
fi
remove_release
mkdir -p "$PREFIX/bin" "$PREFIX/share/pionus" "$PREFIX/share/doc"
install -m 755 "$src/bin/pionus" "$PREFIX/bin/pionus"
for entry in "${RELEASE_FILES[@]}"; do
    cp -R "$src/share/pionus/$entry" "$PREFIX/share/pionus/$entry"
done
cp -R "$src/share/doc/pionus" "$PREFIX/share/doc/pionus"
say "installed $("$PREFIX/bin/pionus" --version) in $PREFIX"
if [ "$UPDATE" = 1 ]; then
    [ -n "$old" ] && say "updated from $old"
    # The new binary looks for the llama.cpp build its release pins; it
    # does nothing when that build is already there.
    if [ -d "${XDG_DATA_HOME:-$HOME/.local/share}/pionus/llama.cpp" ]; then
        "$PREFIX/bin/pionus" setup --llamacpp-only
    fi
fi

# Tells how to put $PREFIX/bin on PATH: the line for the user's shell and the
# file it goes in. The user's files are never edited.
path_note() {
    local dir=$PREFIX/bin file line
    case $dir in
        "$HOME"/*) dir=\$HOME/${dir#"$HOME"/} ;;
    esac
    case $(basename "${SHELL:-}") in
        bash) file=$HOME/.bashrc ;;
        zsh) file=${ZDOTDIR:-$HOME}/.zshrc ;;
        fish) file=$HOME/.config/fish/config.fish ;;
        *) file=$HOME/.profile ;;
    esac
    if [ "$(basename "${SHELL:-}")" = fish ]; then
        line="fish_add_path $dir"
    else
        # shellcheck disable=SC2016  # the $PATH is for the profile, not for this shell
        line="export PATH=\"$dir:"'$PATH"'
    fi
    case $file in
        "$HOME"/*) file=\~/${file#"$HOME"/} ;;
    esac
    echo "note: $PREFIX/bin is not on your PATH, so \`pionus\` is not found yet."
    echo "  Add this line to $file:"
    echo "    $line"
    echo "  Then open a new terminal, or run that line now."
}

case ":$PATH:" in
    *":$PREFIX/bin:"*) ;;
    *) path_note ;;
esac

if [ "$SETUP" = 1 ]; then
    # With a terminal, `pionus setup` opens the full-screen setup (or
    # asks its questions, with --yes it asks none); with curl | bash, stdin is
    # the pipe, so it is given the terminal instead.
    if { : </dev/tty; } 2>/dev/null; then
        "$PREFIX/bin/pionus" setup "${setup_args[@]}" </dev/tty
    else
        "$PREFIX/bin/pionus" setup "${setup_args[@]}"
    fi
elif [ "$UPDATE" = 0 ]; then
    # The name that works in this shell: the bare one only when it is the
    # binary just installed.
    if [ "$(command -v pionus 2>/dev/null)" = "$PREFIX/bin/pionus" ]; then
        next=pionus
    else
        next=$PREFIX/bin/pionus
    fi
    echo "Next: run \`$next setup\` to set up the devices and models."
fi
