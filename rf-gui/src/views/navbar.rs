use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
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
                        "Browse"
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}
