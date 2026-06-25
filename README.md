![Version](https://img.shields.io/badge/version-0.1.1-blue?style=flat-square)
![Rust](https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square&logo=rust)

# Yoru

**Yoru** is a lightweight, modern AUR helper written in Rust for Arch Linux.
It simplifies searching, installing, updating, and managing packages from both the official repositories and the AUR.


## Requirements

- `base-devel`
- `git`
- `rust`

## Install

**From source**
```bash
git clone https://github.com/Crucabena/yoru
cd yoru
cargo build --release
sudo install -Dm755 target/release/yoru /usr/local/bin/yoru
```

## Usage

```bash
yoru -S <pkg>        # install one or more packages
yoru -Ss <query>     # search AUR and official repos
yoru -Syu            # upgrade all packages
yoru -R <pkg>        # remove a package
yoru -Qi <pkg>       # show package info
yoru clean           # clean build cache
yoru doctor          # check system health
```

## Options

| Flag | Description |
|------|-------------|
| `--noconfirm` | Skip confirmation prompts |
| `--needed` | Skip reinstalling up-to-date packages |
| `--all` | Used with `clean` to wipe entire cache |
