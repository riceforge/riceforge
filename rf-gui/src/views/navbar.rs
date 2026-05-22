use crate::{InstalledCount, Route};
use dioxus::desktop::use_window;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    let installed_count: InstalledCount = use_context();
    let window = use_window();

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
                div { class: "navbar-window-controls",
                    button {
                        class: "wc-btn wc-btn--close",
                        title: "Close",
                        onclick: move |_| window.close(),
                        "×"
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}
