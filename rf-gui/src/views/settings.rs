use dioxus::prelude::*;
use rf_core::config::Paths;
use rf_core::index::IndexManager;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, PartialEq)]
enum UpdateStatus {
    Idle,
    Running,
    Done(String),
    Error(String),
}

#[component]
pub fn Settings() -> Element {
    let cache_dir = Paths::cache_dir().display().to_string();
    let data_dir = Paths::data_dir().display().to_string();
    let index_path = Paths::index_cache().display().to_string();
    let index_exists = Paths::index_cache().exists();

    let mut update_status: Signal<UpdateStatus> = use_signal(|| UpdateStatus::Idle);
    let mut cleared = use_signal(|| false);

    rsx! {
        div { class: "settings-page",
            div { class: "settings-header",
                h1 { class: "settings-title", "Settings" }
            }

            div { class: "settings-sections",
                div { class: "settings-section",
                    h2 { class: "settings-section-title", "Directories" }
                    div { class: "settings-row",
                        span { class: "settings-label", "Cache" }
                        span { class: "settings-value", "{cache_dir}" }
                    }
                    div { class: "settings-row",
                        span { class: "settings-label", "Data" }
                        span { class: "settings-value", "{data_dir}" }
                    }
                    div { class: "settings-row",
                        span { class: "settings-label", "Index" }
                        div { class: "settings-row-right",
                            span { class: "settings-value", "{index_path}" }
                            if index_exists {
                                span { class: "settings-badge settings-badge--ok", "cached" }
                            } else {
                                span { class: "settings-badge settings-badge--warn", "not cached" }
                            }
                        }
                    }
                }

                div { class: "settings-section",
                    h2 { class: "settings-section-title", "Index" }
                    p { class: "settings-desc",
                        "The index is fetched from the RiceForge registry and cached locally. "
                        "Update it to see new rices."
                    }
                    div { class: "settings-actions",
                        button {
                            class: "btn-primary",
                            disabled: matches!(update_status(), UpdateStatus::Running),
                            onclick: move |_| {
                                spawn(async move {
                                    update_status.set(UpdateStatus::Running);
                                    let result = tokio::task::spawn_blocking(IndexManager::update).await;
                                    match result {
                                        Ok(Ok(idx)) => update_status.set(UpdateStatus::Done(
                                            format!("{} rices indexed", idx.rices.len())
                                        )),
                                        Ok(Err(e)) => update_status.set(UpdateStatus::Error(e.to_string())),
                                        Err(e) => update_status.set(UpdateStatus::Error(e.to_string())),
                                    }
                                });
                            },
                            if matches!(update_status(), UpdateStatus::Running) {
                                "Updating…"
                            } else {
                                "Update Index"
                            }
                        }

                        if let UpdateStatus::Done(msg) = update_status() {
                            span { class: "settings-status settings-status--ok", "✓ {msg}" }
                        }
                        if let UpdateStatus::Error(msg) = update_status() {
                            span { class: "settings-status settings-status--error", "✗ {msg}" }
                        }
                    }
                }

                div { class: "settings-section",
                    h2 { class: "settings-section-title", "Cache" }
                    p { class: "settings-desc", "Delete the local index cache. It will be re-fetched on next update." }
                    div { class: "settings-actions",
                        button {
                            class: "btn-secondary",
                            disabled: cleared(),
                            onclick: move |_| {
                                let _ = std::fs::remove_file(Paths::index_cache());
                                cleared.set(true);
                            },
                            "Clear Index Cache"
                        }
                        if cleared() {
                            span { class: "settings-status settings-status--ok", "✓ Cache cleared" }
                        }
                    }
                }

                div { class: "settings-section",
                    h2 { class: "settings-section-title", "About" }
                    div { class: "settings-row",
                        span { class: "settings-label", "Version" }
                        span { class: "settings-value", "{VERSION}" }
                    }
                    div { class: "settings-row",
                        span { class: "settings-label", "CLI" }
                        span { class: "settings-value settings-mono", "riceforge --help" }
                    }
                }
            }
        }
    }
}
