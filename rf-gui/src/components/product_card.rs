use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Product {
    pub id: u32,
    pub name: String,
    pub price: f32,
    pub image: String,
    pub description: String,
}

#[component]
pub fn ProductCard(product: Product) -> Element {
    rsx! {
        div {
            class: "product-card",
            div {
                class: "product-image",
                img {
                    src: "{product.image}",
                    alt: "{product.name}",
                }
            }
            div {
                class: "product-info",
                h3 {
                    class: "product-name",
                    "{product.name}"
                }
                p {
                    class: "product-description",
                    "{product.description}"
                }
                div {
                    class: "product-footer",
                    span {
                        class: "product-price",
                        "${product.price}"
                    }
                    button {
                        class: "btn-add",
                        "Добавить"
                    }
                }
            }
        }
    }
}
