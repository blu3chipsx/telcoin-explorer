// src/components/layout.rs — status bar removed, header only
use dioxus::prelude::*;
use crate::components::header::Header;
use crate::router::Route;

const CSS: Asset = asset!("/assets/main.css");

#[component]
pub fn Layout() -> Element {
    rsx! {
        document::Stylesheet { href: CSS }
        div { id: "main",
            Header {}
            Outlet::<Route> {}
        }
    }
}
