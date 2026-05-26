use dioxus::prelude::*;
use crate::components::product_card::{Product, ProductCard};

#[component]
pub fn ProductGrid(products: Vec<Product>) -> Element {
    rsx! {
        div {
            class: "product-grid",
            for product in products {
                ProductCard { key: "{product.id}", product }
            }
        }
    }
}
