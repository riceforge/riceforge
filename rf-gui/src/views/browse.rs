use crate::components::RiceCard;
use dioxus::prelude::*;
use rf_core::{Rice, WindowManager};

fn mock_rices() -> Vec<Rice> {
    vec![
        Rice {
            id: "catppuccin-hyprland".into(),
            name: "Catppuccin Hyprland".into(),
            author: "notashelf".into(),
            description: "Mocha-themed Hyprland setup with Waybar, Kitty and Rofi. Clean, minimal and easy to configure.".into(),
            wm: WindowManager::Hyprland,
            theme: "catppuccin-mocha".into(),
            fonts: vec!["JetBrains Mono".into(), "Noto Sans".into()],
            dependencies: vec!["hyprland".into(), "waybar".into(), "kitty".into(), "rofi".into()],
            repo_url: "https://github.com/notashelf/catppuccin-hyprland".into(),
            screenshots: vec![],
            stars: 342,
            commit_hash: None,
        },
        Rice {
            id: "nord-sway".into(),
            name: "Nord Sway".into(),
            author: "linuxbro".into(),
            description: "Minimalist Nord-themed Sway setup. Swaybar, foot terminal and wofi launcher. Fully keyboard-driven.".into(),
            wm: WindowManager::Sway,
            theme: "nord".into(),
            fonts: vec!["Iosevka".into()],
            dependencies: vec!["sway".into(), "swaybar".into(), "foot".into(), "wofi".into()],
            repo_url: "https://github.com/linuxbro/nord-sway".into(),
            screenshots: vec![],
            stars: 187,
            commit_hash: None,
        },
        Rice {
            id: "gruvbox-i3".into(),
            name: "Gruvbox i3".into(),
            author: "ricemaster".into(),
            description: "Classic Gruvbox i3 setup that stands the test of time. Polybar, urxvt and dmenu.".into(),
            wm: WindowManager::I3,
            theme: "gruvbox-dark".into(),
            fonts: vec!["Hack".into(), "Font Awesome".into()],
            dependencies: vec!["i3-wm".into(), "polybar".into(), "urxvt".into(), "dmenu".into()],
            repo_url: "https://github.com/ricemaster/gruvbox-i3".into(),
            screenshots: vec![],
            stars: 521,
            commit_hash: None,
        },
        Rice {
            id: "tokyo-night-hyprland".into(),
            name: "Tokyo Night".into(),
            author: "0xlain".into(),
            description: "Hyprland setup with Tokyo Night palette. AGS bar, swww wallpapers and smooth animations.".into(),
            wm: WindowManager::Hyprland,
            theme: "tokyo-night".into(),
            fonts: vec!["Geist Mono".into(), "Inter".into()],
            dependencies: vec!["hyprland".into(), "ags".into(), "swww".into(), "wezterm".into()],
            repo_url: "https://github.com/0xlain/tokyo-night-hyprland".into(),
            screenshots: vec![],
            stars: 203,
            commit_hash: None,
        },
        Rice {
            id: "dracula-bspwm".into(),
            name: "Dracula bspwm".into(),
            author: "vampirice".into(),
            description: "Dark Dracula theme for bspwm. Polybar with custom modules, picom blur and alacritty.".into(),
            wm: WindowManager::Bspwm,
            theme: "dracula".into(),
            fonts: vec!["FiraCode Nerd Font".into()],
            dependencies: vec!["bspwm".into(), "sxhkd".into(), "polybar".into(), "picom".into(), "alacritty".into()],
            repo_url: "https://github.com/vampirice/dracula-bspwm".into(),
            screenshots: vec![],
            stars: 98,
            commit_hash: None,
        },
        Rice {
            id: "rosepine-hyprland".into(),
            name: "Rosé Pine".into(),
            author: "prettywm".into(),
            description: "Soft Rosé Pine setup for Hyprland. EWW bar, cava visualizer and a warm pastel palette.".into(),
            wm: WindowManager::Hyprland,
            theme: "rose-pine".into(),
            fonts: vec!["Maple Mono".into(), "Nunito".into()],
            dependencies: vec!["hyprland".into(), "eww".into(), "cava".into(), "kitty".into()],
            repo_url: "https://github.com/prettywm/rosepine-hyprland".into(),
            screenshots: vec![],
            stars: 156,
            commit_hash: None,
        },
    ]
}

#[component]
pub fn Browse() -> Element {
    let mut search = use_signal(|| String::new());
    let filtered = use_memo(move || {
        let q = search().to_lowercase();
        mock_rices()
            .into_iter()
            .filter(|r| {
                q.is_empty()
                    || r.name.to_lowercase().contains(&q)
                    || r.author.to_lowercase().contains(&q)
                    || r.theme.to_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    });

    let rices = filtered();

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
            if rices.is_empty() {
                div { class: "empty-state",
                    h3 { "No rices found" }
                    p { "Try a different search query." }
                }
            } else {
                div { class: "rice-grid",
                    for rice in rices {
                        RiceCard { rice }
                    }
                }
            }
        }
    }
}
