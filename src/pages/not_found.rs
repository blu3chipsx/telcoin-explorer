// src/pages/not_found.rs
use dioxus::prelude::*;
use crate::router::Route;

#[component]
pub fn NotFoundPage(segments: Vec<String>) -> Element {
    let path = segments.join("/");
    rsx! {
        div { class: "page",
            div { class: "error-box",
                style: "text-align: center; padding: 60px;",
                div { style: "font-size: 64px; margin-bottom: 16px;", "404" }
                div { style: "color: var(--text-secondary); margin-bottom: 24px;",
                    "Page not found: /{path}"
                }
                Link { to: Route::HomePage {},
                    button { class: "page-btn", "← Back to Home" }
                }
            }
        }
    }
}
