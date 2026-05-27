use crate::Route;
use dioxus::prelude::*;
use rf_core::installed::InstalledManager;

#[component]
pub fn Navbar() -> Element {
    let installed_count = use_memo(|| InstalledManager::list().map(|l| l.len()).unwrap_or(0));

    rsx! {
        nav { class: "navbar",
            div { class: "navbar-inner",
                Link {
                    to: Route::Browse {},
                    class: "navbar-logo",
                    "RiceForge"
                }
                div { class: "navbar-links",
                    Link {
                        to: Route::Browse {},
                        class: "nav-link",
                        active_class: "nav-link--active",
                        "Browse"
                    }
                    Link {
                        to: Route::Installed {},
                        class: "nav-link",
                        active_class: "nav-link--active",
                        if installed_count() > 0 {
                            "Installed ({installed_count()})"
                        } else {
                            "Installed"
                        }
                    }
                    Link {
                        to: Route::Settings {},
                        class: "nav-link",
                        active_class: "nav-link--active",
                        "Settings"
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}
