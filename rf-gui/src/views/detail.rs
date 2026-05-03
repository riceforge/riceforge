use dioxus::prelude::*;

#[component]
pub fn Detail(id: String) -> Element {
    rsx! {
        div { class: "detail-page",
            p { "Detail for '{id}' — coming soon." }
        }
    }
}
