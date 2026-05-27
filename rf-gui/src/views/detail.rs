use crate::components::rice_card::{thumbnail_gradient, wm_color};
use crate::Route;
use dioxus::prelude::*;
use rf_core::{index::IndexManager, installed::InstalledManager, Rice};

fn load_rice(id: &str) -> Option<Rice> {
    IndexManager::load_cached().ok()?.rices.into_iter().find(|r| r.id == id)
}

#[component]
pub fn Detail(id: String) -> Element {
    let id_clone = id.clone();
    let rice_res = use_resource(move || {
        let id = id_clone.clone();
        async move { load_rice(&id) }
    });

    let id_for_installed = id.clone();
    let installed_res = use_resource(move || {
        let id = id_for_installed.clone();
        async move { InstalledManager::is_installed(&id).unwrap_or(false) }
    });

    match rice_res.read().as_ref() {
        None => rsx! {
            div { class: "detail-page",
                div { class: "detail-loading", "Loading..." }
            }
        },
        Some(None) => rsx! {
            div { class: "detail-page",
                div { class: "detail-not-found",
                    h2 { "Rice not found" }
                    p { "'{id}' does not exist in the index." }
                    Link { to: Route::Browse {}, class: "back-link", "← Back to browse" }
                }
            }
        },
        Some(Some(rice)) => {
            let installed = installed_res.read().as_ref().copied().unwrap_or(false);
            let color = wm_color(&rice.wm);
            let gradient = thumbnail_gradient(&rice.wm);
            let wm_label = rice.wm.to_string();
            let install_cmd = format!("riceforge install {}", rice.id);

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
                                if installed {
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
