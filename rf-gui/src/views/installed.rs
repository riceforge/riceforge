use crate::{InstalledCount, Route};
use dioxus::prelude::*;
use rf_core::{
    InstalledRice,
    deploy::DeployManager,
    git::GitManager,
    index::IndexManager,
    installed::InstalledManager,
    pipeline::{PipelineManager, PipelineWhen},
};

fn load_installed() -> Vec<InstalledRice> {
    InstalledManager::list().unwrap_or_default()
}

fn find_rice_name(id: &str) -> Option<String> {
    IndexManager::load_cached()
        .ok()
        .and_then(|idx| IndexManager::find(&idx, id))
        .map(|r| r.name)
}

fn do_remove(rice_id: String) -> rf_core::Result<()> {
    let index = IndexManager::load_cached()?;
    let rice = index
        .rices
        .into_iter()
        .find(|r| r.id == rice_id)
        .ok_or_else(|| rf_core::RiceForgeError::NotFound(rice_id.clone()))?;

    if let Some(pipeline) = PipelineManager::load(&rice_id)? {
        PipelineManager::run_steps(&pipeline, &PipelineWhen::Remove, &rice_id)?;
    }

    DeployManager::remove(&rice)?;
    InstalledManager::remove(&rice_id)?;
    Ok(())
}

fn do_purge(rice_id: String) -> rf_core::Result<()> {
    do_remove(rice_id.clone())?;
    GitManager::remove(&rice_id)?;
    Ok(())
}

fn do_upgrade(rice_id: String) -> rf_core::Result<String> {
    let index = IndexManager::load_cached()?;
    let rice = index
        .rices
        .into_iter()
        .find(|r| r.id == rice_id)
        .ok_or_else(|| rf_core::RiceForgeError::NotFound(rice_id.clone()))?;

    let commit = GitManager::clone_or_pull(&rice)?;
    let plan = DeployManager::plan(&rice)?;
    DeployManager::apply(&plan)?;
    InstalledManager::add(&rice_id, &commit, None)?;
    Ok(commit)
}

#[derive(Clone, PartialEq)]
enum RowOp {
    Idle,
    Removing,
    Upgrading,
    UpgradeDone(String), // short hash
    Error(String),
}

#[component]
pub fn Installed() -> Element {
    let mut revision = use_signal(|| 0u32);
    let entries = use_memo(move || {
        let _ = revision();
        load_installed()
    });

    rsx! {
        div { class: "installed-page",
            div { class: "installed-header",
                h1 { class: "installed-title", "Installed Rices" }
                span { class: "installed-count", "{entries().len()} installed" }
            }

            if entries().is_empty() {
                div { class: "empty-state",
                    h3 { "Nothing installed yet" }
                    p { "Browse rices and install one to get started." }
                    Link { to: Route::Browse {}, class: "btn-primary", "Browse" }
                }
            } else {
                div { class: "installed-list",
                    {
                        let all = entries();
                        all.into_iter().map(|entry| rsx! {
                            InstalledRow {
                                key: "{entry.rice_id}",
                                entry: entry.clone(),
                                on_removed: move || {
                                    *revision.write() += 1;
                                },
                            }
                        })
                    }
                }
            }
        }
    }
}

#[component]
fn InstalledRow(entry: InstalledRice, on_removed: EventHandler<()>) -> Element {
    let rice_id = entry.rice_id.clone();
    let rice_id_remove = entry.rice_id.clone();
    let rice_id_purge  = entry.rice_id.clone();
    let rice_id_upgrade = entry.rice_id.clone();

    let display_name = find_rice_name(&rice_id).unwrap_or_else(|| rice_id.clone());
    let short_hash = entry
        .commit_hash
        .get(..8)
        .unwrap_or(&entry.commit_hash)
        .to_string();
    let date = entry.installed_at.format("%Y-%m-%d").to_string();

    let mut op: Signal<RowOp> = use_signal(|| RowOp::Idle);
    let mut confirm_purge = use_signal(|| false);
    let mut installed_count: InstalledCount = use_context();

    rsx! {
        div { class: "installed-row",
            div { class: "installed-row-info",
                div { class: "installed-row-header",
                    Link {
                        to: Route::Detail { id: rice_id.clone() },
                        class: "installed-row-name",
                        "{display_name}"
                    }
                    span { class: "installed-row-id", "{rice_id}" }
                }
                div { class: "installed-row-meta",
                    span { class: "installed-row-hash",
                        if let RowOp::UpgradeDone(ref h) = op() {
                            "{h}"
                        } else {
                            "{short_hash}"
                        }
                    }
                    span { class: "installed-row-date", "installed {date}" }
                    if entry.backup_id.is_some() {
                        span { class: "installed-row-backup", "backup available" }
                    }
                }
            }

            div { class: "installed-row-actions",
                if matches!(op(), RowOp::Idle | RowOp::UpgradeDone(_)) && !confirm_purge() {
                    // Upgrade
                    button {
                        class: "btn-ghost btn-sm",
                        title: "Pull latest commit and redeploy symlinks",
                        onclick: move |_| {
                            let id = rice_id_upgrade.clone();
                            spawn(async move {
                                op.set(RowOp::Upgrading);
                                let result = tokio::task::spawn_blocking(move || do_upgrade(id)).await;
                                match result {
                                    Ok(Ok(commit)) => {
                                        let short = commit.get(..8).unwrap_or(&commit).to_string();
                                        op.set(RowOp::UpgradeDone(short));
                                    }
                                    Ok(Err(e)) => op.set(RowOp::Error(e.to_string())),
                                    Err(e)     => op.set(RowOp::Error(e.to_string())),
                                }
                            });
                        },
                        "Upgrade"
                    }
                    // Remove
                    button {
                        class: "btn-secondary btn-sm",
                        onclick: move |_| {
                            let id = rice_id_remove.clone();
                            let on_removed = on_removed;
                            spawn(async move {
                                op.set(RowOp::Removing);
                                let result = tokio::task::spawn_blocking(move || do_remove(id)).await;
                                match result {
                                    Ok(Ok(())) => {
                                        let count = InstalledManager::list()
                                            .map(|l| l.len())
                                            .unwrap_or(0);
                                        installed_count.set(count);
                                        on_removed.call(());
                                    }
                                    Ok(Err(e)) => op.set(RowOp::Error(e.to_string())),
                                    Err(e)     => op.set(RowOp::Error(e.to_string())),
                                }
                            });
                        },
                        "Remove"
                    }
                    // Purge trigger
                    button {
                        class: "btn-ghost btn-sm",
                        onclick: move |_| confirm_purge.set(true),
                        "Purge"
                    }
                }

                if confirm_purge() {
                    div { class: "purge-confirm",
                        span { "Delete repo too?" }
                        button {
                            class: "btn-danger btn-sm",
                            onclick: move |_| {
                                let id = rice_id_purge.clone();
                                let on_removed = on_removed;
                                spawn(async move {
                                    op.set(RowOp::Removing);
                                    let result = tokio::task::spawn_blocking(move || do_purge(id)).await;
                                    match result {
                                        Ok(Ok(())) => {
                                            let count = InstalledManager::list()
                                                .map(|l| l.len())
                                                .unwrap_or(0);
                                            installed_count.set(count);
                                            on_removed.call(());
                                        }
                                        Ok(Err(e)) => op.set(RowOp::Error(e.to_string())),
                                        Err(e)     => op.set(RowOp::Error(e.to_string())),
                                    }
                                });
                            },
                            "Yes, purge"
                        }
                        button {
                            class: "btn-ghost btn-sm",
                            onclick: move |_| confirm_purge.set(false),
                            "Cancel"
                        }
                    }
                }

                if matches!(op(), RowOp::Removing) {
                    span { class: "row-op-status", "Removing…" }
                }
                if matches!(op(), RowOp::Upgrading) {
                    span { class: "row-op-status", "Upgrading…" }
                }
                if let RowOp::UpgradeDone(_) = op() {
                    span { class: "row-op-status row-op-status--ok", "✓ Updated" }
                }
                if let RowOp::Error(msg) = op() {
                    span { class: "row-op-status row-op-status--error", "{msg}" }
                }
            }
        }
    }
}
