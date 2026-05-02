use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    rsx! {
        div {
            class: "header",
            div {
                class: "header-content",
                div {
                    class: "header-info",
                    h1 { "Магазин" }
                    p { "Выберите идеальный товар для себя" }
                }
                div {
                    class: "search-box",
                    input {
                        r#type: "text",
                        placeholder: "Поиск товаров...",
                        class: "search-input"
                    }
                }
            }
        }
    }
}
