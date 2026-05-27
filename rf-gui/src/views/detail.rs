use crate::components::rice_card::{thumbnail_gradient, wm_color};
use crate::Route;
use dioxus::prelude::*;
use rf_core::{index::IndexManager, installed::InstalledManager, Rice};

fn find_rice(id: &str) -> Option<Rice> {
    IndexManager::load_cached()
        .ok()
        .and_then(|idx| IndexManager::find(&idx, id))
}

#[component]
pub fn Detail(id: String) -> Element {
    let id_rice = id.clone();
    let id_installed = id.clone();

    let rice = use_memo(move || find_rice(&id_rice));
    let installed = use_memo(move || {
        InstalledManager::is_installed(&id_installed).unwrap_or(false)
    });

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

            rsx! {
                div { class: "detail-page",
                    Link { to: Route::Browse {}, class: "back-link", "← Browse" }

                    div { class: "detail-hero",
                        div {
                            class: "detail-thumbnail",
                            style: "background: {gradient}",
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
                            }
                        }
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
                            h3 { class: "section-title", "Install" }
                            div { class: "code-block",
                                code { "{install_cmd}" }
                            }
                        }

                        if is_installed {
                            div { class: "detail-section",
                                h3 { class: "section-title", "Remove" }
                                div { class: "code-block",
                                    code { "riceforge remove {rice.id}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
