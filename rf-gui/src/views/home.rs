const SHOP_CSS: Asset = asset!("/assets/styling/shop.css");
use dioxus::prelude::*;
use crate::components::{Header, ProductGrid, product_card::Product};

/// The Home page component - shop interface in Nothing Phone / Grok style
#[component]
pub fn Home() -> Element {
    // Sample products for demonstration
    let products = vec![
        Product {
            id: 1,
            name: "Премиум наушники".to_string(),
            price: 199.99,
            image: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 200 200'%3E%3Crect fill='%23f5f5f5' width='200' height='200'/%3E%3Ccircle cx='100' cy='100' r='60' fill='%23e0e0e0'/%3E%3C/svg%3E".to_string(),
            description: "Высочайшее качество звука с активным шумоподавлением".to_string(),
        },
        Product {
            id: 2,
            name: "Беспроводная мышь".to_string(),
            price: 79.99,
            image: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 200 200'%3E%3Crect fill='%23f5f5f5' width='200' height='200'/%3E%3Cellipse cx='100' cy='80' rx='40' ry='50' fill='%23e0e0e0'/%3E%3C/svg%3E".to_string(),
            description: "Эргономичный дизайн с точной сенсорной системой".to_string(),
        },
        Product {
            id: 3,
            name: "Механическая клавиатура".to_string(),
            price: 149.99,
            image: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 200 200'%3E%3Crect fill='%23f5f5f5' width='200' height='200'/%3E%3Crect x='40' y='60' width='120' height='80' fill='%23e0e0e0'/%3E%3C/svg%3E".to_string(),
            description: "RGB подсветка, программируемые клавиши".to_string(),
        },
        Product {
            id: 4,
            name: "USB-C Хаб".to_string(),
            price: 59.99,
            image: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 200 200'%3E%3Crect fill='%23f5f5f5' width='200' height='200'/%3E%3Crect x='50' y='70' width='100' height='60' rx='10' fill='%23e0e0e0'/%3E%3C/svg%3E".to_string(),
            description: "7-в-1 многофункциональный адаптер".to_string(),
        },
        Product {
            id: 5,
            name: "Портативный аккумулятор".to_string(),
            price: 89.99,
            image: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 200 200'%3E%3Crect fill='%23f5f5f5' width='200' height='200'/%3E%3Crect x='60' y='50' width='80' height='100' rx='8' fill='%23e0e0e0'/%3E%3C/svg%3E".to_string(),
            description: "65000 мАч, поддержка быстрой зарядки".to_string(),
        },
        Product {
            id: 6,
            name: "4K Веб-камера".to_string(),
            price: 129.99,
            image: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 200 200'%3E%3Crect fill='%23f5f5f5' width='200' height='200'/%3E%3Ccircle cx='100' cy='100' r='50' fill='%23e0e0e0'/%3E%3C/svg%3E".to_string(),
            description: "Идеальна для стриминга и видеоконференций".to_string(),
        },
    ];

    rsx! {
        document::Link { rel: "stylesheet", href: SHOP_CSS }
        
        div {
            class: "shop-container",
            Header {}
            ProductGrid { products }
        }
    }
}
