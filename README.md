# RiceForge

A decentralized app store for Linux dotfile configurations (rices). Browse, install, and manage window manager setups — Hyprland, Sway, i3, bspwm, and more.

## Features

- **Serverless registry** — index lives in GitHub, no backend needed
- **CLI and GUI** — full-featured terminal interface + native desktop app (Dioxus)
- **Stow-style deployment** — symlinks from cloned repos into `~`, clean removal
- **Automatic backups** — backs up existing configs before deploying
- **Pipeline support** — optional `pipeline.toml` for post-install scripts
- **Package resolution** — detects and installs missing pacman packages
- **Zero telemetry** — no tracking, no accounts, no forced registration

## Installation

### Arch Linux / CachyOS

**CLI only:**

```bash
git clone https://github.com/riceforge/riceforge
cd riceforge
cargo build --release -p riceforge
sudo install -Dm755 target/release/riceforge /usr/local/bin/riceforge
riceforge update
```

**GUI (requires webkit2gtk):**

```bash
sudo pacman -S --needed webkit2gtk-4.1 gtk3 libsoup3
cargo build --release -p rf-gui
./target/release/rf-gui
```

On Wayland/Hyprland the window is borderless by default — drag it with `Super+drag`.

## CLI Usage

```bash
riceforge update                        # fetch/refresh the index
riceforge search <query> [--wm hyprland] [--theme nord]
riceforge list [--installed]
riceforge info <rice-id>
riceforge install <rice-id> [--dry-run] [--no-packages]
riceforge remove <rice-id> [--restore] [--purge]
riceforge backup list
riceforge backup restore <id>
riceforge backup clean [N]              # keep N most recent (default 5)
```

## Rice format

Each rice is a GitHub repository with a `rice.toml` at the root:

```toml
id          = "catppuccin-hyprland"
name        = "Catppuccin Hyprland"
author      = "notashelf"
description = "Mocha-themed Hyprland setup with Waybar and Kitty."
wm          = "hyprland"
theme       = "catppuccin-mocha"
fonts       = ["JetBrains Mono", "Noto Sans"]
dependencies = ["hyprland", "waybar", "kitty", "rofi"]
repo_url    = "https://github.com/notashelf/catppuccin-hyprland"
screenshots = ["screenshots/preview.png"]
```

Optional `pipeline.toml` for post-install/pre-remove scripts:

```toml
[[steps]]
name = "Enable waybar"
run  = "systemctl --user enable --now waybar.service"
when = "install"   # install | remove | always
```

## How it works

1. `riceforge update` downloads `index.json` from the registry and caches it at `~/.cache/riceforge/`
2. `riceforge install <id>` clones the rice repo to `~/.local/share/riceforge/rices/<id>/`
3. Files are symlinked from the repo into `~` (Stow strategy)
4. Existing configs are backed up to `~/.local/share/riceforge/backups/`
5. `riceforge remove <id>` removes symlinks; `--restore` restores the backup

## Submitting a rice

1. Create a public GitHub repository with your dotfiles
2. Add `rice.toml` to the root (see [examples/rice.toml](examples/rice.toml))
3. Open a PR to [riceforge/riceforge-index](https://github.com/riceforge/riceforge-index)

## Architecture

```
rf-core    — business logic (git, deploy, backup, packages, pipeline)
rf-cli     — CLI binary (riceforge)
rf-gui     — desktop GUI (Dioxus 0.7)
rf-index   — index builder/validator (GitHub Actions)
```

## Development

```bash
./scripts/build.sh     # release build
./scripts/test.sh      # run all tests
./scripts/dev.sh       # watch mode
```

Requires Rust stable (1.80+). On Arch: `sudo pacman -S rust`.

## License

MIT
