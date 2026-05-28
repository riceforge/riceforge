use crate::models::WindowManager;

pub fn detect_wm() -> Option<WindowManager> {
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return Some(WindowManager::Hyprland);
    }

    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_default();

    wm_from_str(&desktop)
}

fn wm_from_str(s: &str) -> Option<WindowManager> {
    let s = s.to_lowercase();
    if s.contains("hyprland") {
        Some(WindowManager::Hyprland)
    } else if s.contains("sway") {
        Some(WindowManager::Sway)
    } else if s.contains("i3") {
        Some(WindowManager::I3)
    } else if s.contains("openbox") {
        Some(WindowManager::Openbox)
    } else if s.contains("bspwm") {
        Some(WindowManager::Bspwm)
    } else if s.contains("qtile") {
        Some(WindowManager::Qtile)
    } else if s.contains("xmonad") {
        Some(WindowManager::Xmonad)
    } else {
        None
    }
}
