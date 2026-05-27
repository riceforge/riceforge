# RiceForge

[![CI](https://github.com/riceforge/riceforge/actions/workflows/ci.yml/badge.svg)](https://github.com/riceforge/riceforge/actions/workflows/ci.yml)

Децентрализованный менеджер dotfiles для Linux. Позволяет находить, устанавливать и управлять конфигурациями оконных менеджеров — Hyprland, Sway, i3, bspwm и других — из терминала или через графический интерфейс, без серверов и без телеметрии.

---

## Содержание

- [Возможности](#возможности)
- [Архитектура](#архитектура)
- [Установка](#установка)
- [Первый запуск](#первый-запуск)
- [Справочник CLI](#справочник-cli)
- [Графический интерфейс](#графический-интерфейс)
- [Формат rice.toml](#формат-ricetoml)
- [Формат pipeline.toml](#формат-pipelinetoml)
- [Как работает установка](#как-работает-установка)
- [Файловая структура](#файловая-структура)
- [Добавление своего rice](#добавление-своего-rice)
- [Разработка](#разработка)

---

## Возможности

- **Serverless-реестр** — индекс хранится на GitHub как статический `index.json`, без собственных серверов и баз данных
- **CLI и GUI** — полноценный терминальный интерфейс (`riceforge`) и нативное desktop-приложение (`rf-gui`) на Dioxus
- **Stow-развёртывание** — симлинки из клонированного репозитория напрямую в домашний каталог, чистое удаление одной командой
- **Автоматические бэкапы** — перед любыми изменениями `~/.config` сохраняется снапшот с возможностью мгновенного отката
- **Pipeline-скрипты** — опциональный `pipeline.toml` в dotfiles-репозитории для пост-установочных и пре-удаления команд
- **Разрешение зависимостей** — автоматически определяет недостающие пакеты pacman, устанавливает их через `sudo` (CLI) или `pkexec` (GUI)
- **Обнаружение WM** — определяет запущенный оконный менеджер и показывает совместимость прямо в интерфейсе
- **Без телеметрии** — никакого трекинга, аккаунтов и принудительной регистрации

---

## Архитектура

RiceForge состоит из двух репозиториев и четырёх Rust-крейтов в одном Cargo workspace.

### Репозитории

```
riceforge/riceforge        — основное приложение (этот репозиторий)
riceforge/riceforge-index  — реестр: rice.toml файлы + сгенерированный index.json
github.com/*/dotfiles      — dotfiles-репозитории авторов (внешние, не форкнуты)
```

### Крейты

```
rf-core    — вся бизнес-логика: Git, развёртывание, бэкапы, пакеты, pipeline
rf-cli     — консольный бинарник riceforge, тонкая обёртка над rf-core
rf-gui     — desktop-интерфейс на Dioxus 0.7 + WebKit2GTK
rf-index   — утилита для сборки и валидации index.json (используется в GitHub Actions)
```

### Поток данных

```
Автор                     GitHub Actions              Пользователь
  │                            │                           │
  ├─ публикует dotfiles ──────►│                           │
  ├─ открывает PR в ──────────►│ validate.yml              │
  │  riceforge-index           │  └─ rf-index validate     │
  │                            │                           │
  ├─ PR смёржен ──────────────►│ build.yml                 │
  │                            │  └─ rf-index build        │
  │                            │      └─► index.json       │
  │                            │                           │
  │                (ежедневно) │ update-stars.yml          │
  │                            │  └─ rf-index update-stars │
  │                            │      └─► index.json       │
  │                            │                           │
  │                            │              riceforge update
  │                            │           ←──── curl ─────┤
  │                            │                           │
  │                            │              riceforge install <id>
  │                            │                    git clone
  │                            │               ←──────────┤
  │                            │                    symlinks
  │                            │                    pacman -S
  │                            │                    pipeline.toml
  │                            │                           │
```

### Взаимодействие крейтов

```
rf-cli ──────┐
             ├──► rf-core ──► Git (git2 / git subprocess)
rf-gui ──────┘               ──► Deploy (symlinks)
                             ──► BackupManager
                             ──► PackageManager (pacman)
                             ──► IndexManager (curl + JSON)
                             ──► PipelineManager (sh)
                             ──► InstalledManager (installed.json)

rf-index ──► (standalone, используется только в CI)
```

---

## Установка

### Системные требования

- Linux (Arch, CachyOS или любой дистрибутив с pacman)
- `curl` в PATH
- `git` в PATH
- Для GUI: `webkit2gtk-4.1`, `gtk3`, работающий polkit-агент (для установки пакетов через pkexec)

### Из бинарного релиза

Открыть страницу [Releases](https://github.com/riceforge/riceforge/releases/latest) и скачать нужный архив.

**CLI:**

```bash
wget https://github.com/riceforge/riceforge/releases/latest/download/riceforge-v0.1.0-linux-x86_64.tar.gz
tar xzf riceforge-v0.1.0-linux-x86_64.tar.gz
sudo install -Dm755 riceforge /usr/local/bin/riceforge
```

**GUI:**

```bash
# Системные зависимости
sudo pacman -S --needed webkit2gtk-4.1 gtk3

wget https://github.com/riceforge/riceforge/releases/latest/download/rf-gui-v0.1.0-linux-x86_64.tar.gz
tar xzf rf-gui-v0.1.0-linux-x86_64.tar.gz
sudo install -Dm755 rf-gui /usr/local/bin/rf-gui
```

### Сборка из исходников

```bash
# Зависимости для сборки
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

Требуемая версия Rust: stable 1.80 и выше.

---

## Первый запуск

```bash
# 1. Загрузить индекс (обязательный шаг перед любой работой)
riceforge update

# 2. Посмотреть доступные конфигурации
riceforge list

# 3. Найти что-нибудь конкретное
riceforge search hyprland --wm hyprland

# 4. Посмотреть подробности
riceforge info jakoolit-hyprland

# 5. Установить
riceforge install jakoolit-hyprland
```

---

## Справочник CLI

### `riceforge update`

Загружает актуальный `index.json` с GitHub и сохраняет в локальный кэш (`~/.cache/riceforge/index.json`). Использует `curl -sf --connect-timeout 15`. Все остальные команды читают только кэш — сети не требуют.

```bash
riceforge update
# ✓ 42 rices indexed (updated 2025-05-27)
```

---

### `riceforge search [QUERY] [OPTIONS]`

Локальный поиск по кэшированному индексу. Без аргументов — показывает все rice. Поиск выполняется по полям `id`, `name`, `author`, `description`, `theme`.

| Флаг | Описание |
|---|---|
| `--wm`, `-w` | Фильтр по оконному менеджеру (`hyprland`, `sway`, `i3`, `bspwm`, `qtile`, `xmonad`, `openbox`) |
| `--theme`, `-t` | Фильтр по теме (частичное совпадение, без учёта регистра) |

```bash
riceforge search                           # всё
riceforge search catppuccin                # по ключевому слову
riceforge search --wm hyprland            # только Hyprland
riceforge search nord --theme nord        # Nord-темы
```

Установленные rice отмечаются `✓` в начале строки.

---

### `riceforge list [--installed]`

Выводит список всех доступных rice из кэша. `✓` обозначает установленные.

| Флаг | Описание |
|---|---|
| `--installed` | Показывать только установленные rice с хэшем текущего коммита |

```bash
riceforge list
riceforge list --installed
# ✓ jakoolit-hyprland  a3f1c82d
```

---

### `riceforge info <ID>`

Показывает подробную информацию: автор, WM, тема, звёзды, URL репозитория, описание, статус установки, список зависимостей с отметкой об их наличии в системе, шрифты.

```bash
riceforge info jakoolit-hyprland
```

Пример вывода:

```
JaKooLit Hyprland
  id          jakoolit-hyprland
  author      JaKooLit
  wm          hyprland
  theme       catppuccin-mocha
  stars       2341
  repo        https://github.com/JaKooLit/Hyprland-Dots
  status      not installed

  A feature-rich Hyprland configuration with Catppuccin Mocha ...

  dependencies
    ✓ hyprland
    ✗ waybar
    ✓ kitty
```

---

### `riceforge install <ID> [OPTIONS]`

Полный цикл установки: клонирование → бэкап → установка пакетов → симлинки → pipeline.

| Флаг | Описание |
|---|---|
| `--dry-run` | Показать план изменений без применения |
| `--no-packages` | Не устанавливать пакеты pacman |
| `--force` | Переустановить, даже если уже установлен; перезаписать конфликтующие симлинки |

**Шаги, выполняемые при установке:**

1. Клонирование репозитория в `~/.local/share/riceforge/rices/<id>/` (или `git pull`, если уже клонирован)
2. Построение плана развёртывания — список симлинков, конфликтов, файлов для бэкапа
3. Вывод плана для ознакомления (всегда, перед применением)
4. Обнаружение конфликтов (симлинк уже ведёт в другой rice)
5. Создание бэкапа существующих файлов из `~/.config` в `~/.local/share/riceforge/backups/<timestamp>/`
6. Установка недостающих пакетов через `sudo pacman -S --needed --noconfirm`
7. Создание симлинков
8. Запись в `installed.json`
9. Выполнение шагов `pipeline.toml` с `when = "install"` или `when = "always"`
10. Автоочистка бэкапов (оставляет 5 последних)

```bash
riceforge install jakoolit-hyprland
riceforge install jakoolit-hyprland --dry-run
riceforge install jakoolit-hyprland --no-packages --force
```

---

### `riceforge upgrade [ID] [--all]`

Обновляет клонированный репозиторий до последнего коммита и переприменяет симлинки.

| Флаг | Описание |
|---|---|
| `--all` | Обновить все установленные rice |

```bash
riceforge upgrade jakoolit-hyprland
riceforge upgrade --all
```

---

### `riceforge remove <ID> [OPTIONS]`

Удаляет симлинки развёрнутого rice. Перед удалением выполняет шаги `pipeline.toml` с `when = "remove"` или `when = "always"`.

| Флаг | Описание |
|---|---|
| `--restore` | После удаления симлинков восстановить последний бэкап |
| `--purge` | Дополнительно удалить клонированный репозиторий из `rices/` |

```bash
riceforge remove jakoolit-hyprland
riceforge remove jakoolit-hyprland --restore
riceforge remove jakoolit-hyprland --restore --purge
```

---

### `riceforge check`

Проверяет целостность симлинков всех установленных rice. Для каждого — проверяет, что симлинки существуют и указывают на правильный источник в `rices_dir`.

```bash
riceforge check
# ✓ jakoolit-hyprland  14 symlinks OK
# ✗ my-rice            2/8 broken
#     /home/user/.config/waybar/config
```

---

### `riceforge backup <SUBCOMMAND>`

Управление снапшотами конфигурации.

| Подкоманда | Описание |
|---|---|
| `list` | Показать все бэкапы: ID, rice, количество файлов |
| `restore <ID>` | Восстановить конкретный бэкап |
| `clean [N]` | Удалить все бэкапы, кроме N последних (по умолчанию N=5) |

```bash
riceforge backup list
# 20250527_143012  jakoolit-hyprland  7 files
# 20250526_091533  my-rice            3 files

riceforge backup restore 20250527_143012
riceforge backup clean 3
```

Бэкапы содержат только файлы из `~/.config`, существовавшие до установки и не являющиеся симлинками. Каждый бэкап хранит `meta.json` с метаданными.

---

## Графический интерфейс

```bash
rf-gui
```

GUI предоставляет те же возможности, что и CLI, в визуальном виде:

- **Список rice** с превью скриншотов, WM-бейджами и счётчиком звёзд
- **Поиск и фильтрация** в реальном времени по названию, WM, теме
- **Страница rice** с галереей скриншотов, описанием, зависимостями и статусом совместимости с текущим WM
- **Установка** с живым отображением прогресса Git (каждая строка вывода `git clone`)
- **Управление пакетами** — определяет недостающие зависимости, предлагает установить через PolicyKit (графическое окно ввода пароля, без необходимости открывать терминал)

GUI определяет текущий WM через переменные среды (`HYPRLAND_INSTANCE_SIGNATURE`, `XDG_CURRENT_DESKTOP`, `DESKTOP_SESSION`) и показывает, совместим ли конкретный rice с вашим окружением.

---

## Формат rice.toml

Каждый rice описывается файлом `rice.toml` в репозитории `riceforge-index`. Сам dotfiles-репозиторий не требует изменений.

```toml
id           = "my-hyprland"
name         = "My Hyprland"
author       = "yourusername"
description  = "Минималистичная конфигурация Hyprland с темой Catppuccin Mocha (20–300 символов)."
wm           = "hyprland"
theme        = "catppuccin-mocha"
fonts        = ["JetBrains Mono Nerd Font", "Noto Sans"]
dependencies = ["hyprland", "waybar", "kitty", "rofi-wayland"]
repo_url     = "https://github.com/yourusername/dotfiles"
screenshots  = ["https://raw.githubusercontent.com/yourusername/dotfiles/main/preview.png"]
```

| Поле | Обязательно | Описание |
|---|---|---|
| `id` | ✅ | Уникальный идентификатор. Только буквы, цифры, `-`, `_`. Без пробелов. |
| `name` | ✅ | Отображаемое название. |
| `author` | ✅ | GitHub-имя автора. |
| `description` | ✅ | От 20 до 300 символов. |
| `wm` | ✅ | Один из: `hyprland`, `sway`, `i3`, `bspwm`, `qtile`, `xmonad`, `openbox` |
| `theme` | ✅ | Название цветовой схемы (произвольная строка). |
| `repo_url` | ✅ | Должен начинаться с `https://github.com/`. |
| `fonts` | — | Список шрифтов. Отображается в интерфейсе, не устанавливается автоматически. |
| `dependencies` | — | Пакеты pacman. Проверяются через `pacman -Q`, устанавливаются при `install`. |
| `screenshots` | — | URL изображений для превью. Также автоматически берутся из папки `screenshots/` в `riceforge-index`. |

---

## Формат pipeline.toml

Опциональный файл `pipeline.toml` размещается в корне **dotfiles-репозитория** (не в `riceforge-index`). Позволяет выполнять команды после установки или перед удалением.

```toml
[[steps]]
name = "Enable waybar"
run  = "systemctl --user enable --now waybar.service"
when = "install"

[[steps]]
name = "Enable hypridle"
run  = "systemctl --user enable --now hypridle.service"
when = "install"

[[steps]]
name = "Disable waybar"
run  = "systemctl --user disable --now waybar.service"
when = "remove"

[[steps]]
name = "Reload shell"
run  = "source ~/.bashrc"
when = "always"
```

| Поле | Тип | Описание |
|---|---|---|
| `name` | string | Человекочитаемое название шага (выводится в интерфейсе) |
| `run` | string | Команда, выполняемая через `sh -c` |
| `when` | `install` / `remove` / `always` | Фаза выполнения. По умолчанию: `always` |

Шаги выполняются с рабочим каталогом `~/.local/share/riceforge/rices/<id>/`. Если какой-либо шаг завершается с ненулевым кодом — установка/удаление прерывается с ошибкой.

---

## Как работает установка

### Стратегия развёртывания (Stow-style)

При установке `riceforge` клонирует dotfiles-репозиторий в:

```
~/.local/share/riceforge/rices/<rice-id>/
```

Затем рекурсивно обходит все файлы в этой директории и создаёт соответствующие симлинки в домашнем каталоге. Файлы из корня репозитория, перечисленные ниже, исключаются:

```
rice.toml    pipeline.toml    README.md    README.rst    README
LICENSE      LICENSE.md       LICENSE.txt  .git          .gitignore
.gitmodules  screenshots/     preview.png  preview.jpg   preview.webp
INSTALL.md   install.sh
```

Пример: файл `~/.local/share/riceforge/rices/my-rice/.config/hypr/hyprland.conf` создаёт симлинк `~/.config/hypr/hyprland.conf`.

### Обнаружение конфликтов

Перед применением симлинков проверяется:

- **Конфликт**: целевой путь уже является симлинком, ведущим в другой rice. В этом случае установка останавливается (до `--force`).
- **Файл под бэкап**: целевой путь является обычным файлом или директорией — он будет скопирован в бэкап перед заменой.

### Бэкапы

Снапшоты создаются только из файлов `~/.config`. Каждый бэкап — это директория вида:

```
~/.local/share/riceforge/backups/20250527_143012/
├── meta.json          # ID, rice_id, дата, список файлов
├── hypr/
│   └── hyprland.conf
└── waybar/
    └── config
```

---

## Файловая структура

```
~/.cache/riceforge/
└── index.json                     # кэш реестра (~/.cache по XDG)

~/.local/share/riceforge/
├── installed.json                 # база установленных rice
├── rices/
│   ├── jakoolit-hyprland/         # клонированный dotfiles-репозиторий
│   │   ├── .config/
│   │   │   ├── hypr/
│   │   │   └── waybar/
│   │   └── pipeline.toml
│   └── my-rice/
└── backups/
    ├── 20250527_143012/
    │   ├── meta.json
    │   └── hypr/hyprland.conf
    └── 20250526_091533/
        └── meta.json
```

`installed.json` содержит массив записей с полями `rice_id`, `commit_hash`, `installed_at`, `backup_id`.

---

## Добавление своего rice

1. Создайте публичный GitHub-репозиторий с dotfiles
2. Опционально: добавьте `pipeline.toml` для пост-установочных команд
3. Форкните [riceforge/riceforge-index](https://github.com/riceforge/riceforge-index)
4. Создайте папку `rices/<your-id>/` и добавьте `rice.toml`
5. Опционально: добавьте скриншоты в `rices/<your-id>/screenshots/`
6. Откройте Pull Request — CI автоматически проверит `rice.toml`
7. После успешных проверок PR можно мёржить; `index.json` пересобирается автоматически

---

## Разработка

### Сборка

```bash
./scripts/build.sh       # сборка релизных бинарников всех крейтов
./scripts/test.sh        # cargo test --all
./scripts/dev.sh         # cargo watch -x test (авторестарт при изменении файлов)
```

### Структура workspace

```
Cargo.toml               # workspace root, resolver = "3"
rf-core/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── models.rs        # Rice, Index, WindowManager, BackupEntry, DeployPlan
    ├── config.rs        # Paths (cache_dir, data_dir, rices_dir, ...)
    ├── index.rs         # IndexManager (update, load_cached, search, find)
    ├── git.rs           # GitManager (clone_or_pull, clone_or_pull_with_progress)
    ├── deploy.rs        # DeployManager (plan, apply, remove)
    ├── backup.rs        # BackupManager (create, list, restore, clean)
    ├── packages.rs      # PackageManager (is_installed, missing, install, install_gui)
    ├── pipeline.rs      # PipelineManager (load, run_steps)
    ├── installed.rs     # InstalledManager (add, remove, list, get, is_installed)
    ├── system.rs        # detect_wm()
    └── error.rs         # RiceForgeError, Result
rf-cli/
├── Cargo.toml
└── src/main.rs
rf-gui/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── components/
    └── views/
rf-index/
├── Cargo.toml
└── src/main.rs
fixtures/
├── test-index.json
└── test-rice-*/
```

### Тестирование

Unit-тесты находятся в каждом модуле `rf-core`. Интеграционные тесты — в `rf-core/tests/`. Фикстуры — в `fixtures/`.

```bash
cargo test -p rf-core                  # только core
cargo test -p rf-core index            # конкретный модуль
```

---

## Лицензия

[MIT](LICENSE)
