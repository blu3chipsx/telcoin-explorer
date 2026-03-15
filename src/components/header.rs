// src/components/header.rs
use dioxus::prelude::*;
use crate::router::Route;

const LOGO: Asset = asset!("/assets/telcoin-logo.svg");

#[component]
pub fn Header() -> Element {
    rsx! {
        header { class: "header",
            div { class: "header-inner",
                Link { to: Route::HomePage {},
                    div { class: "logo",
                        img { src: LOGO, class: "logo-img", alt: "Telcoin" }
                        div { class: "logo-text-wrap",
                            span { class: "logo-name", "TelScan" }
                            span { class: "logo-badge", "Adiri Testnet" }
                        }
                    }
                }
                div { style: "flex:1;" }
                nav { class: "header-nav",
                    Link { to: Route::HomePage {}, "Home" }
                    Link { to: Route::ValidatorsPage {}, "Validators" }
                    a {
                        href: "https://telcoin.network/faucet",
                        target: "_blank",
                        class: "header-nav-faucet",
                        "Faucet ↗"
                    }
                }
            }
        }
    }
}
