# RiceForge

[![CI](https://github.com/riceforge/riceforge/actions/workflows/ci.yml/badge.svg)](https://github.com/riceforge/riceforge/actions/workflows/ci.yml)

Децентрализованный менеджер dotfiles для Linux. Находите, устанавливайте и управляйте конфигурациями оконных менеджеров — Hyprland, Sway, i3, bspwm и других — прямо с терминала или через GUI.

## Возможности

- **Serverless-реестр** — индекс хранится на GitHub, без собственного сервера
- **CLI и GUI** — полноценный терминальный интерфейс и нативное desktop-приложение (Dioxus)
- **Stow-развёртывание** — симлинки из клонированного репо в `~`, чистое удаление
- **Автоматические бэкапы** — существующие конфиги сохраняются перед установкой
- **Pipeline-скрипты** — опциональный `pipeline.toml` для пост-установочных команд
- **Разрешение зависимостей** — определяет и предлагает установить недостающие пакеты
- **Без телеметрии** — никакого трекинга, аккаунтов или обязательной регистрации

## Архитектура

```
riceforge/riceforge          — основное приложение (этот репозиторий)
riceforge/riceforge-index    — реестр: rice.toml файлы + index.json
github.com/*/dotfiles        — dotfiles-репозитории авторов (внешние)
```

```
rf-core    — бизнес-логика (git, deploy, backup, packages, pipeline)
rf-cli     — CLI-бинарник (riceforge)
rf-gui     — desktop-интерфейс (Dioxus 0.7 + WebKit)
rf-index   — сборщик и валидатор реестра (GitHub Actions)
```

Как это работает:

1. Автор публикует dotfiles на GitHub, открывает PR в `riceforge-index` с `rice.toml`
2. CI автоматически валидирует метаданные, после мёржа пересобирает `index.json`
3. Пользователь: `riceforge update` → `riceforge install <id>` → конфиги на месте

## Установка

### Из бинарного релиза (рекомендуется)

Открыть страницу [releases](https://github.com/riceforge/riceforge/releases/latest) и скачать нужный архив.

**CLI:**

```bash
wget https://github.com/riceforge/riceforge/releases/latest/download/riceforge-v0.1.0-linux-x86_64.tar.gz
tar xzf riceforge-v0.1.0-linux-x86_64.tar.gz
sudo install -Dm755 riceforge /usr/local/bin/riceforge
sudo install -Dm755 rf-index  /usr/local/bin/rf-index
```

**GUI:**

```bash
# Сначала установить системные зависимости
sudo pacman -S --needed webkit2gtk-4.1 gtk3

wget https://github.com/riceforge/riceforge/releases/latest/download/rf-gui-v0.1.0-linux-x86_64.tar.gz
tar xzf rf-gui-v0.1.0-linux-x86_64.tar.gz
sudo install -Dm755 rf-gui /usr/local/bin/rf-gui
```

### Сборка из исходников

```bash
# Зависимости
sudo pacman -S --needed rust git

git clone https://github.com/riceforge/riceforge
cd riceforge

# CLI
cargo build --release -p rf-cli
sudo install -Dm755 target/release/riceforge /usr/local/bin/riceforge

# GUI (требует webkit2gtk-4.1)
sudo pacman -S --needed webkit2gtk-4.1 gtk3
cargo build --release -p rf-gui
sudo install -Dm755 target/release/rf-gui /usr/local/bin/rf-gui
```

## Использование CLI

```bash
riceforge update                              # обновить локальный кэш реестра
riceforge list                                # список всех доступных rice
riceforge list --installed                    # только установленные
riceforge search <запрос>                     # поиск по названию/автору
riceforge search <запрос> --wm hyprland       # с фильтром по WM
riceforge info <id>                           # подробная информация
riceforge install <id>                        # установить rice
riceforge install <id> --dry-run              # предварительный просмотр без изменений
riceforge install <id> --no-packages          # без авто-установки пакетов
riceforge install <id> --force                # переустановить поверх существующего
riceforge upgrade <id>                        # обновить до последнего коммита
riceforge upgrade --all                       # обновить все установленные
riceforge remove <id>                         # удалить симлинки
riceforge remove <id> --restore               # удалить + восстановить бэкап
riceforge remove <id> --purge                 # удалить + удалить клонированный репо
riceforge check                               # проверить целостность симлинков
riceforge backup list                         # список бэкапов
riceforge backup restore <id>                 # восстановить бэкап
riceforge backup clean [N]                    # удалить старые (оставить N последних)
```

## Формат rice.toml

Каждый rice — это GitHub-репозиторий с метаданными в `riceforge-index`. Метаданные описываются файлом `rice.toml`:

```toml
id           = "my-hyprland"
name         = "My Hyprland"
author       = "username"
description  = "Краткое описание (20–300 символов)."
wm           = "hyprland"           # hyprland | sway | i3 | bspwm | qtile | xmonad | openbox
theme        = "catppuccin-mocha"
fonts        = ["JetBrains Mono Nerd Font"]
dependencies = ["hyprland", "waybar", "kitty"]
repo_url     = "https://github.com/username/dotfiles"
screenshots  = ["https://raw.githubusercontent.com/username/dotfiles/main/preview.png"]
```

Опциональный `pipeline.toml` в dotfiles-репозитории для пост-установочных/пре-удаления скриптов:

```toml
[[steps]]
name = "Enable waybar"
run  = "systemctl --user enable --now waybar.service"
when = "install"   # install | remove | always
```

## Добавление своего rice

1. Создайте публичный GitHub-репозиторий с dotfiles
2. Откройте PR в [riceforge/riceforge-index](https://github.com/riceforge/riceforge-index) с файлом `rice.toml`
3. CI автоматически проверит метаданные — при успехе PR можно мёржить

## Структура данных

```
~/.cache/riceforge/
└── index.json                    # кэш реестра

~/.local/share/riceforge/
├── installed.json                # база установленных rice
├── rices/
│   └── <rice-id>/               # клонированный dotfiles-репозиторий
└── backups/
    └── <timestamp>/             # снапшот ~/.config до установки
```

## Разработка

```bash
./scripts/build.sh    # сборка релизных бинарников
./scripts/test.sh     # все тесты
./scripts/dev.sh      # watch-режим (cargo watch)
```

Требования: Rust stable 1.80+. На Arch: `sudo pacman -S rust`.

## Лицензия

[MIT](LICENSE)
