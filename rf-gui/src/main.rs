use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use views::{Browse, Detail, Installed, Navbar, Settings};

mod components;
mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Browse {},
        #[route("/rice/:id")]
        Detail { id: String },
        #[route("/installed")]
        Installed {},
        #[route("/settings")]
        Settings {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::default().with_window(
                WindowBuilder::new()
                    .with_decorations(false)
                    .with_title(concat!("RiceForge v", env!("CARGO_PKG_VERSION"))),
            ),
        )
        .launch(App);
}

pub type InstalledCount = Signal<usize>;

#[component]
fn App() -> Element {
    let initial = rf_core::installed::InstalledManager::list()
        .map(|l| l.len())
        .unwrap_or(0);
    use_context_provider(|| Signal::new(initial));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}
