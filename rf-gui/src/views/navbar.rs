use crate::Route;
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

/// The Navbar component that will be rendered on all pages of our app since every page is under the layout.
#[component]
pub fn Navbar() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        div {
            id: "navbar",
            div {
                class: "navbar-container",
                Link {
                    to: Route::Home {},
                    class: "navbar-logo",
                    "Shop"
                }
                div {
                    class: "navbar-links",
                    a {
                        href: "#",
                        class: "nav-link",
                        "Каталог"
                    }
                    a {
                        href: "#",
                        class: "nav-link",
                        "О нас"
                    }
                    a {
                        href: "#",
                        class: "nav-link",
                        "Контакты"
                    }
                }
                button {
                    class: "nav-cart",
                    "🛒 Корзина"
                }
            }
        }

        // The `Outlet` component is used to render the next component inside the layout. In this case, it will render
        // the [`Home`] component depending on the current route.
        Outlet::<Route> {}
    }
}
