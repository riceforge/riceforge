use crate::Route;
use dioxus::prelude::*;
use rf_core::{Rice, WindowManager};

pub fn wm_color(wm: &WindowManager) -> &'static str {
    match wm {
        WindowManager::Hyprland => "#a855f7",
        WindowManager::Sway => "#3b82f6",
        WindowManager::I3 => "#22c55e",
        WindowManager::Bspwm => "#f97316",
        WindowManager::Qtile => "#ec4899",
        WindowManager::Xmonad => "#eab308",
        WindowManager::Openbox => "#06b6d4",
        WindowManager::Unknown => "#71717a",
    }
}

pub fn thumbnail_gradient(wm: &WindowManager) -> &'static str {
    match wm {
        WindowManager::Hyprland => "linear-gradient(135deg, #180d2e 0%, #2d1654 100%)",
        WindowManager::Sway => "linear-gradient(135deg, #0a1628 0%, #0e2a4a 100%)",
        WindowManager::I3 => "linear-gradient(135deg, #0d1f0d 0%, #143314 100%)",
        WindowManager::Bspwm => "linear-gradient(135deg, #1e0a00 0%, #3d1a00 100%)",
        WindowManager::Qtile => "linear-gradient(135deg, #1e0020 0%, #3d0040 100%)",
        WindowManager::Xmonad => "linear-gradient(135deg, #1a1600 0%, #332d00 100%)",
        WindowManager::Openbox => "linear-gradient(135deg, #001e1e 0%, #003535 100%)",
        WindowManager::Unknown => "linear-gradient(135deg, #111111 0%, #1a1a1a 100%)",
    }
}

#[component]
pub fn RiceCard(rice: Rice) -> Element {
    let color = wm_color(&rice.wm);
    let gradient = thumbnail_gradient(&rice.wm);
    let wm_label = rice.wm.to_string();
    let id = rice.id.clone();

    rsx! {
        Link {
            to: Route::Detail { id },
            class: "rice-card",
            div {
                class: "rice-thumbnail",
                style: "background: {gradient}",
                div {
                    class: "rice-wm-badge",
                    style: "color: {color}; border-color: {color}",
                    "{wm_label}"
                }
            }
            div { class: "rice-info",
                div { class: "rice-header",
                    span { class: "rice-name", "{rice.name}" }
                    span { class: "rice-stars", "★ {rice.stars}" }
                }
                span { class: "rice-author", "@{rice.author}" }
                p { class: "rice-description", "{rice.description}" }
                div { class: "rice-footer",
                    div { class: "rice-tags",
                        span { class: "rice-tag", "{rice.theme}" }
                    }
                    span { class: "install-hint", "view →" }
                }
            }
        }
    }
}
