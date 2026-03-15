// src/components/layout.rs
use dioxus::prelude::*;
use crate::components::header::Header;
use crate::components::footer::Footer;

const CSS: Asset = asset!("/assets/main.css");

#[component]
pub fn Layout() -> Element {
    rsx! {
        document::Stylesheet { href: CSS }
        div { class: "app-wrap",
            Header {}
            main { class: "app-main",
                Outlet::<crate::router::Route> {}
            }
            Footer {}
        }
    }
}
