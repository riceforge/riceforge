use crate::{InstalledCount, Route};
use crate::components::rice_card::{thumbnail_gradient, wm_color};
use dioxus::prelude::*;
use rf_core::{
    DeployPlan, Rice,
    backup::BackupManager,
    deploy::DeployManager,
    git::GitManager,
    index::IndexManager,
    installed::InstalledManager,
    packages::PackageManager,
    pipeline::{PipelineManager, PipelineWhen},
};

#[derive(Clone, PartialEq)]
enum PkgState {
    Idle,
    Installing,
    Done,
    Error(String),
}

fn copy_to_clipboard(text: &str) {
    use std::io::Write;
    if let Ok(mut child) = std::process::Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
        return;
    }
    if let Ok(mut child) = std::process::Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

fn find_rice(id: &str) -> Option<Rice> {
    IndexManager::load_cached()
        .ok()
        .and_then(|idx| IndexManager::find(&idx, id))
}

#[derive(Clone, PartialEq)]
enum InstallState {
    Idle,
    Planning,
    ConfirmPlan {
        links: Vec<String>,
        to_backup: Vec<String>,
        has_pipeline: bool,
        missing_pkgs: Vec<String>,
        conflicts: Vec<String>,
    },
    Applying,
    Done(String),
    Error(String),
}

#[derive(Clone, PartialEq)]
enum RemoveState {
    Idle,
    Confirm,
    Removing,
    Done,
    Error(String),
}

fn do_plan(
    rice: Rice,
    tx: std::sync::mpsc::Sender<String>,
) -> rf_core::Result<(DeployPlan, bool, Vec<String>, Vec<String>)> {
    GitManager::clone_or_pull_with_progress(&rice, move |line| {
        let _ = tx.send(line);
    })?;
    let plan = DeployManager::plan(&rice)?;
    let has_pipeline = PipelineManager::load(&rice.id)?.is_some();
    let missing_pkgs = if PackageManager::is_available() {
        PackageManager::missing(&rice.dependencies)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![]
    };
    let conflicts = plan
        .conflicts
        .iter()
        .map(|(dest, _)| dest.display().to_string())
        .collect();
    Ok((plan, has_pipeline, missing_pkgs, conflicts))
}

fn do_apply(rice: Rice, tx: std::sync::mpsc::Sender<String>) -> rf_core::Result<String> {
    let commit = GitManager::clone_or_pull_with_progress(&rice, move |line| {
        let _ = tx.send(line);
    })?;
    let plan = DeployManager::plan(&rice)?;

    let backup_id = if !plan.to_backup.is_empty() {
        let entry = BackupManager::create(Some(&rice.id), &plan.to_backup)?;
        Some(entry.id)
    } else {
        None
    };

    DeployManager::apply(&plan)?;
    InstalledManager::add(&rice.id, &commit, backup_id)?;

    if let Some(pipeline) = PipelineManager::load(&rice.id)? {
        PipelineManager::run_steps(&pipeline, &PipelineWhen::Install, &rice.id)?;
    }

    Ok(commit)
}

fn do_remove(rice: Rice) -> rf_core::Result<()> {
    if let Some(pipeline) = PipelineManager::load(&rice.id)? {
        PipelineManager::run_steps(&pipeline, &PipelineWhen::Remove, &rice.id)?;
    }
    DeployManager::remove(&rice)?;
    InstalledManager::remove(&rice.id)?;
    Ok(())
}

#[component]
pub fn Detail(id: String) -> Element {
    let id_rice = id.clone();
    let id_installed = id.clone();

    let rice = use_memo(move || find_rice(&id_rice));

    let mut installed =
        use_signal(move || InstalledManager::is_installed(&id_installed).unwrap_or(false));
    let mut install_state: Signal<InstallState> = use_signal(|| InstallState::Idle);
    let mut remove_state: Signal<RemoveState> = use_signal(|| RemoveState::Idle);
    let mut copied: Signal<bool> = use_signal(|| false);
    let mut git_progress: Signal<String> = use_signal(String::new);
    let mut pkg_state: Signal<PkgState> = use_signal(|| PkgState::Idle);
    let mut installed_count: InstalledCount = use_context();

    // Detect the current WM once; compute compat info eagerly as plain data
    let user_wm_name: Option<String> = rf_core::detect_wm().map(|w| w.to_string());

    match rice() {
        None => rsx! {
            div { class: "detail-page",
                Link { to: Route::Browse {}, class: "back-link", "← Browse" }
                div { class: "detail-not-found",
                    h2 { "Rice not found" }
                    p { "'{id}' does not exist in the index." }
                    p { "Run " code { "riceforge update" } " to refresh the index." }
                }
            }
        },
        Some(rice) => {
            let color = wm_color(&rice.wm);
            let gradient = thumbnail_gradient(&rice.wm);
            let wm_label = rice.wm.to_string();
            let install_cmd = format!("riceforge install {}", rice.id);
            let is_installed = installed();
            let is_busy = matches!(
                install_state(),
                InstallState::Planning | InstallState::Applying
            ) || matches!(remove_state(), RemoveState::Removing);

            // Button label for the "Install Packages" button — computed reactively
            let pkg_btn_label = match pkg_state() {
                PkgState::Idle => "Install Packages",
                PkgState::Installing => "Installing…",
                PkgState::Done => "✓ Installed",
                PkgState::Error(_) => "Retry",
            };

            // WM compatibility: None = unknown WM, Some(true/false) = match/mismatch
            let rice_wm_str = rice.wm.to_string();
            let wm_compat: Option<bool> = user_wm_name
                .as_deref()
                .map(|u| u.to_lowercase() == rice_wm_str.to_lowercase());

            let rice_for_plan = rice.clone();
            let rice_for_apply = rice.clone();
            let rice_for_remove = rice.clone();

            // Hero thumbnail: use <img> overlay for reliable rendering
            let hero_bg = format!("background: {gradient};");
            let hero_img = rice.screenshots.first().cloned();

            rsx! {
                div { class: "detail-page",
                    Link { to: Route::Browse {}, class: "back-link", "← Browse" }

                    div { class: "detail-hero",
                        div {
                            class: "detail-thumbnail",
                            style: "{hero_bg}",
                            if let Some(url) = hero_img {
                                img {
                                    class: "detail-thumbnail-img",
                                    src: url,
                                    loading: "eager",
                                }
                            }
                            div {
                                class: "rice-wm-badge",
                                style: "color: {color}; border-color: {color}",
                                "{wm_label}"
                            }
                        }
                        div { class: "detail-meta",
                            div { class: "detail-header",
                                h1 { class: "detail-name", "{rice.name}" }
                                if is_installed {
                                    span { class: "installed-badge", "installed" }
                                }
                            }
                            p { class: "detail-author", "@{rice.author}" }
                            div { class: "detail-stats",
                                span { class: "detail-stat", "★ {rice.stars}" }
                                span { class: "detail-stat", "{rice.theme}" }
                            }
                            p { class: "detail-description", "{rice.description}" }

                            // WM compatibility indicator
                            if wm_compat == Some(true) {
                                div { class: "wm-compat wm-compat--ok",
                                    "✓ Compatible with your window manager"
                                }
                            }
                            if wm_compat == Some(false) {
                                div { class: "wm-compat wm-compat--warn",
                                    "⚠ Designed for {wm_label} — may not work on your current setup"
                                }
                            }

                            div { class: "detail-actions",
                                a {
                                    class: "btn-secondary",
                                    href: "{rice.repo_url}",
                                    "View on GitHub"
                                }

                                if !is_installed && !matches!(install_state(), InstallState::Done(_)) {
                                    button {
                                        class: "btn-primary",
                                        disabled: is_busy,
                                        onclick: move |_| {
                                            let rice = rice_for_plan.clone();
                                            spawn(async move {
                                                install_state.set(InstallState::Planning);
                                                git_progress.set(String::new());

                                                let (tx, rx) = std::sync::mpsc::channel::<String>();
                                                let rice_clone = rice.clone();
                                                let handle = tokio::task::spawn_blocking(move || {
                                                    do_plan(rice_clone, tx)
                                                });

                                                // Poll progress while clone/pull runs
                                                while !handle.is_finished() {
                                                    if let Ok(line) = rx.try_recv() {
                                                        git_progress.set(line);
                                                    }
                                                    tokio::time::sleep(
                                                        std::time::Duration::from_millis(100)
                                                    ).await;
                                                }
                                                // Drain remaining messages
                                                while let Ok(line) = rx.try_recv() {
                                                    git_progress.set(line);
                                                }

                                                match handle.await {
                                                    Ok(Ok((plan, has_pipeline, missing_pkgs, conflicts))) => {
                                                        let links = plan.links.iter().map(|(_, d)| {
                                                            d.display().to_string()
                                                        }).collect();
                                                        let to_backup = plan.to_backup.iter().map(|p| {
                                                            p.display().to_string()
                                                        }).collect();
                                                        git_progress.set(String::new());
                                                        install_state.set(InstallState::ConfirmPlan {
                                                            links,
                                                            to_backup,
                                                            has_pipeline,
                                                            missing_pkgs,
                                                            conflicts,
                                                        });
                                                    }
                                                    Ok(Err(e)) => install_state.set(InstallState::Error(e.to_string())),
                                                    Err(e) => install_state.set(InstallState::Error(e.to_string())),
                                                }
                                            });
                                        },
                                        if matches!(install_state(), InstallState::Planning) {
                                            "Preparing…"
                                        } else {
                                            "Install"
                                        }
                                    }
                                }

                                if is_installed {
                                    if matches!(remove_state(), RemoveState::Idle | RemoveState::Error(_)) {
                                        button {
                                            class: "btn-danger",
                                            disabled: is_busy,
                                            onclick: move |_| remove_state.set(RemoveState::Confirm),
                                            "Remove"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Screenshots gallery (remaining screenshots after the first)
                    if rice.screenshots.len() > 1 {
                        div { class: "detail-screenshots",
                            for url in rice.screenshots.iter() {
                                img {
                                    class: "detail-screenshot",
                                    src: url.clone(),
                                    loading: "lazy",
                                }
                            }
                        }
                    }

                    // Git progress (shown during Planning and Applying)
                    if matches!(install_state(), InstallState::Planning | InstallState::Applying) {
                        div { class: "op-status op-status--running",
                            if matches!(install_state(), InstallState::Planning) {
                                "Cloning repository…"
                            } else {
                                "Installing…"
                            }
                            {
                                let prog = git_progress();
                                if !prog.is_empty() {
                                    rsx! { div { class: "git-progress-line", "{prog}" } }
                                } else {
                                    None
                                }
                            }
                        }
                    }

                    // Install plan confirmation
                    if let InstallState::ConfirmPlan { links, to_backup, has_pipeline, missing_pkgs, conflicts } = install_state() {
                        div { class: "plan-box",
                            h3 { class: "plan-title", "Deploy Plan" }

                            if !conflicts.is_empty() {
                                div { class: "conflict-box",
                                    p { class: "conflict-title",
                                        "⚠ Conflict — these files belong to another rice:"
                                    }
                                    for path in &conflicts {
                                        div { class: "plan-file plan-file--conflict", "{path}" }
                                    }
                                    p { class: "conflict-hint",
                                        "Remove the other rice first, or the conflicting symlinks will be overwritten."
                                    }
                                }
                            }

                            if !missing_pkgs.is_empty() {
                                {
                                    let pkgs_str = missing_pkgs.join(" ");
                                    let pacman_cmd = format!("sudo pacman -S --needed {pkgs_str}");
                                    let cmd_for_copy = pacman_cmd.clone();
                                    let pkgs_for_install = missing_pkgs.clone();
                                    rsx! {
                                        div { class: "missing-pkgs",
                                            p { class: "missing-pkgs-title",
                                                "⚠ Missing packages — install before proceeding:"
                                            }
                                            div { class: "missing-pkgs-chips",
                                                for pkg in &missing_pkgs {
                                                    span { class: "dep-chip dep-chip--missing", "{pkg}" }
                                                }
                                            }
                                            div { class: "missing-pkgs-cmd-row",
                                                code { class: "missing-pkgs-cmd", "{pacman_cmd}" }
                                                button {
                                                    class: if copied() { "btn-ghost btn-sm copied" } else { "btn-ghost btn-sm" },
                                                    onclick: move |_| {
                                                        if copied() { return; }
                                                        let cmd = cmd_for_copy.clone();
                                                        spawn(async move {
                                                            tokio::task::spawn_blocking(move || {
                                                                copy_to_clipboard(&cmd);
                                                            }).await.ok();
                                                            copied.set(true);
                                                            tokio::time::sleep(
                                                                std::time::Duration::from_secs(2)
                                                            ).await;
                                                            copied.set(false);
                                                        });
                                                    },
                                                    if copied() { "Copied!" } else { "Copy" }
                                                }
                                            }

                                            // One-click install via pkexec (shows polkit auth dialog)
                                            div { class: "missing-pkgs-install-row",
                                                button {
                                                    class: "btn-primary btn-sm",
                                                    disabled: matches!(pkg_state(), PkgState::Installing | PkgState::Done),
                                                    onclick: move |_| {
                                                        let pkgs = pkgs_for_install.clone();
                                                        spawn(async move {
                                                            pkg_state.set(PkgState::Installing);
                                                            let result = tokio::task::spawn_blocking(move || {
                                                                PackageManager::install_gui(&pkgs)
                                                            }).await;
                                                            match result {
                                                                Ok(Ok(())) => pkg_state.set(PkgState::Done),
                                                                Ok(Err(e)) => pkg_state.set(PkgState::Error(e.to_string())),
                                                                Err(e)    => pkg_state.set(PkgState::Error(e.to_string())),
                                                            }
                                                        });
                                                    },
                                                    "{pkg_btn_label}"
                                                }
                                                span { class: "pkg-install-hint",
                                                    "Uses pkexec — a password dialog will appear"
                                                }
                                            }

                                            if let PkgState::Error(msg) = pkg_state() {
                                                p { class: "pkg-install-error", "{msg}" }
                                            }
                                        }
                                    }
                                }
                            }

                            p { class: "plan-desc", "{links.len()} symlink(s) will be created in your home directory." }

                            if !to_backup.is_empty() {
                                div { class: "plan-section",
                                    p { class: "plan-section-label", "Files to back up first:" }
                                    for f in &to_backup {
                                        div { class: "plan-file plan-file--backup", "{f}" }
                                    }
                                }
                            }

                            div { class: "plan-section",
                                p { class: "plan-section-label", "Symlinks to create:" }
                                for dest in links.iter().take(12) {
                                    div { class: "plan-file", "{dest}" }
                                }
                                if links.len() > 12 {
                                    div { class: "plan-file plan-file--more",
                                        "… and {links.len() - 12} more"
                                    }
                                }
                            }

                            if has_pipeline {
                                p { class: "plan-pipeline-note",
                                    "This rice includes a pipeline.toml — post-install scripts will run."
                                }
                            }

                            div { class: "plan-actions",
                                button {
                                    class: if conflicts.is_empty() { "btn-primary" } else { "btn-danger" },
                                    onclick: move |_| {
                                        let rice = rice_for_apply.clone();
                                        spawn(async move {
                                            install_state.set(InstallState::Applying);
                                            git_progress.set(String::new());

                                            let (tx, rx) = std::sync::mpsc::channel::<String>();
                                            let rice_clone = rice.clone();
                                            let handle = tokio::task::spawn_blocking(move || {
                                                do_apply(rice_clone, tx)
                                            });

                                            while !handle.is_finished() {
                                                if let Ok(line) = rx.try_recv() {
                                                    git_progress.set(line);
                                                }
                                                tokio::time::sleep(
                                                    std::time::Duration::from_millis(100)
                                                ).await;
                                            }
                                            while let Ok(line) = rx.try_recv() {
                                                git_progress.set(line);
                                            }

                                            match handle.await {
                                                Ok(Ok(commit)) => {
                                                    installed.set(true);
                                                    let count = InstalledManager::list()
                                                        .map(|l| l.len())
                                                        .unwrap_or(0);
                                                    installed_count.set(count);
                                                    let short = commit.get(..8).unwrap_or(&commit).to_string();
                                                    git_progress.set(String::new());
                                                    install_state.set(InstallState::Done(short));
                                                }
                                                Ok(Err(e)) => install_state.set(InstallState::Error(e.to_string())),
                                                Err(e) => install_state.set(InstallState::Error(e.to_string())),
                                            }
                                        });
                                    },
                                    "Confirm Install"
                                }
                                button {
                                    class: "btn-secondary",
                                    onclick: move |_| install_state.set(InstallState::Idle),
                                    "Cancel"
                                }
                            }
                        }
                    }

                    if let InstallState::Done(hash) = install_state() {
                        div { class: "op-status op-status--done", "Installed at commit {hash}" }
                    }

                    if let InstallState::Error(msg) = install_state() {
                        div { class: "op-status op-status--error", "Error: {msg}" }
                    }

                    // Remove confirmation
                    if matches!(remove_state(), RemoveState::Confirm) {
                        div { class: "plan-box plan-box--danger",
                            h3 { class: "plan-title", "Remove Rice" }
                            p { class: "plan-desc",
                                "All symlinks created by this rice will be removed. Your backup (if any) will be preserved."
                            }
                            div { class: "plan-actions",
                                button {
                                    class: "btn-danger",
                                    onclick: move |_| {
                                        let rice = rice_for_remove.clone();
                                        spawn(async move {
                                            remove_state.set(RemoveState::Removing);
                                            let result = tokio::task::spawn_blocking(move || {
                                                do_remove(rice)
                                            }).await;
                                            match result {
                                                Ok(Ok(())) => {
                                                    installed.set(false);
                                                    let count = InstalledManager::list()
                                                        .map(|l| l.len())
                                                        .unwrap_or(0);
                                                    installed_count.set(count);
                                                    remove_state.set(RemoveState::Done);
                                                }
                                                Ok(Err(e)) => remove_state.set(RemoveState::Error(e.to_string())),
                                                Err(e) => remove_state.set(RemoveState::Error(e.to_string())),
                                            }
                                        });
                                    },
                                    "Confirm Remove"
                                }
                                button {
                                    class: "btn-secondary",
                                    onclick: move |_| remove_state.set(RemoveState::Idle),
                                    "Cancel"
                                }
                            }
                        }
                    }

                    if matches!(remove_state(), RemoveState::Removing) {
                        div { class: "op-status op-status--running", "Removing…" }
                    }

                    if matches!(remove_state(), RemoveState::Done) {
                        div { class: "op-status op-status--done", "Rice removed successfully." }
                    }

                    if let RemoveState::Error(msg) = remove_state() {
                        div { class: "op-status op-status--error", "Error: {msg}" }
                    }

                    div { class: "detail-sections",
                        if !rice.dependencies.is_empty() {
                            div { class: "detail-section",
                                h3 { class: "section-title", "Dependencies" }
                                div { class: "deps-list",
                                    for dep in &rice.dependencies {
                                        span { class: "dep-chip", "{dep}" }
                                    }
                                }
                            }
                        }

                        if !rice.fonts.is_empty() {
                            div { class: "detail-section",
                                h3 { class: "section-title", "Fonts" }
                                div { class: "deps-list",
                                    for font in &rice.fonts {
                                        span { class: "dep-chip", "{font}" }
                                    }
                                }
                            }
                        }

                        div { class: "detail-section",
                            h3 { class: "section-title", "Install via CLI" }
                            div { class: "code-block",
                                code { "{install_cmd}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
