use crate::{Route, components::RiceCard};
use dioxus::prelude::*;
use rf_core::Rice;

fn load_rices() -> Result<Vec<Rice>, String> {
    rf_core::index::IndexManager::load_cached()
        .map(|idx| idx.rices)
        .map_err(|e| e.to_string())
}

#[component]
pub fn Browse() -> Element {
    let mut search = use_signal(String::new);
    let mut wm_filter: Signal<Option<String>> = use_signal(|| None);

    // Async load so the UI thread is never blocked and we can show skeletons
    let load_result = use_resource(|| async move {
        tokio::task::spawn_blocking(load_rices)
            .await
            .unwrap_or_else(|e| Err(e.to_string()))
    });

    let filtered = use_memo(move || {
        let Some(Ok(all)) = load_result.read().as_ref().cloned() else {
            return vec![];
        };
        let q = search().to_lowercase();
        let wm = wm_filter();

        all.into_iter()
            .filter(|r| {
                let matches_q = q.is_empty()
                    || r.name.to_lowercase().contains(&q)
                    || r.author.to_lowercase().contains(&q)
                    || r.theme.to_lowercase().contains(&q)
                    || r.id.to_lowercase().contains(&q);

                let matches_wm = wm
                    .as_deref()
                    .is_none_or(|w| r.wm.to_string().to_lowercase() == w);

                matches_q && matches_wm
            })
            .collect::<Vec<_>>()
    });

    let wm_options: &[(&str, Option<&str>)] = &[
        ("All", None),
        ("Hyprland", Some("hyprland")),
        ("Sway", Some("sway")),
        ("i3", Some("i3")),
        ("bspwm", Some("bspwm")),
        ("Qtile", Some("qtile")),
    ];

    rsx! {
        div { class: "browse-page",
            div { class: "browse-header",
                h1 { class: "browse-title", "Browse Rices" }
                input {
                    class: "search-input",
                    r#type: "text",
                    placeholder: "Search by name, author or theme...",
                    value: search,
                    oninput: move |e| *search.write() = e.value(),
                }
            }

            match load_result.read().as_ref() {
                // Still loading — show skeleton cards
                None => rsx! {
                    div { class: "wm-filters skeleton-filters",
                        for _ in 0..5 {
                            div { class: "skeleton skeleton-chip" }
                        }
                    }
                    div { class: "rice-grid",
                        for _ in 0..8 {
                            div { class: "rice-card skeleton-card",
                                div { class: "skeleton rice-thumbnail" }
                                div { class: "rice-info",
                                    div { class: "skeleton skeleton-line skeleton-line--title" }
                                    div { class: "skeleton skeleton-line skeleton-line--sub" }
                                    div { class: "skeleton skeleton-line skeleton-line--desc" }
                                    div { class: "skeleton skeleton-line skeleton-line--desc skeleton-line--short" }
                                }
                            }
                        }
                    }
                },
                // Index not available
                Some(Err(_)) => rsx! {
                    div { class: "empty-state",
                        h3 { "Index not loaded" }
                        p { "Open Settings and click " strong { "Update Index" } " to fetch the rice registry." }
                        Link { to: Route::Settings {}, class: "btn-primary", "Go to Settings" }
                    }
                },
                // Loaded OK
                Some(Ok(_)) => rsx! {
                    div { class: "wm-filters",
                        for &(label, value) in wm_options {
                            button {
                                class: if wm_filter().as_deref() == value {
                                    "wm-chip wm-chip--active"
                                } else {
                                    "wm-chip"
                                },
                                onclick: move |_| {
                                    *wm_filter.write() = value.map(|s| s.to_string());
                                },
                                "{label}"
                            }
                        }
                    }

                    if filtered().is_empty() {
                        div { class: "empty-state",
                            h3 { "No rices found" }
                            p { "Try a different search or filter." }
                        }
                    } else {
                        div { class: "rice-grid",
                            for rice in filtered() {
                                RiceCard { rice }
                            }
                        }
                    }
                },
            }
        }
    }
}
