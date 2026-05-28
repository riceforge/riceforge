use crate::Route;
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

fn do_plan(rice: Rice) -> rf_core::Result<(DeployPlan, bool, Vec<String>)> {
    GitManager::clone_or_pull(&rice)?;
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
    Ok((plan, has_pipeline, missing_pkgs))
}

fn do_apply(rice: Rice) -> rf_core::Result<String> {
    let commit = GitManager::clone_or_pull(&rice)?;
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

            let rice_for_plan = rice.clone();
            let rice_for_apply = rice.clone();
            let rice_for_remove = rice.clone();

            let hero_style = if let Some(url) = rice.screenshots.first() {
                format!(
                    "background: {gradient}; background-image: url('{url}'); background-size: cover; background-position: center;"
                )
            } else {
                format!("background: {gradient};")
            };

            rsx! {
                div { class: "detail-page",
                    Link { to: Route::Browse {}, class: "back-link", "← Browse" }

                    div { class: "detail-hero",
                        div {
                            class: "detail-thumbnail",
                            style: "{hero_style}",
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
                                                let result = tokio::task::spawn_blocking(move || {
                                                    do_plan(rice)
                                                }).await;
                                                match result {
                                                    Ok(Ok((plan, has_pipeline, missing_pkgs))) => {
                                                        let links = plan.links.iter().map(|(_, d)| {
                                                            d.display().to_string()
                                                        }).collect();
                                                        let to_backup = plan.to_backup.iter().map(|p| {
                                                            p.display().to_string()
                                                        }).collect();
                                                        install_state.set(InstallState::ConfirmPlan {
                                                            links,
                                                            to_backup,
                                                            has_pipeline,
                                                            missing_pkgs,
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

                    // Screenshots gallery
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

                    // Install plan confirmation
                    if let InstallState::ConfirmPlan { links, to_backup, has_pipeline, missing_pkgs } = install_state() {
                        div { class: "plan-box",
                            h3 { class: "plan-title", "Deploy Plan" }

                            if !missing_pkgs.is_empty() {
                                {
                                    let pkgs_str = missing_pkgs.join(" ");
                                    let pacman_cmd = format!("sudo pacman -S --needed {pkgs_str}");
                                    let cmd_for_copy = pacman_cmd.clone();
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
                                                    class: "btn-ghost btn-sm",
                                                    onclick: move |_| {
                                                        let cmd = cmd_for_copy.clone();
                                                        spawn(async move {
                                                            tokio::task::spawn_blocking(move || {
                                                                copy_to_clipboard(&cmd);
                                                            }).await.ok();
                                                        });
                                                    },
                                                    "Copy"
                                                }
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
                                    class: "btn-primary",
                                    onclick: move |_| {
                                        let rice = rice_for_apply.clone();
                                        spawn(async move {
                                            install_state.set(InstallState::Applying);
                                            let result = tokio::task::spawn_blocking(move || {
                                                do_apply(rice)
                                            }).await;
                                            match result {
                                                Ok(Ok(commit)) => {
                                                    installed.set(true);
                                                    let short = commit.get(..8).unwrap_or(&commit).to_string();
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

                    if matches!(install_state(), InstallState::Applying) {
                        div { class: "op-status op-status--running", "Installing… this may take a moment" }
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
