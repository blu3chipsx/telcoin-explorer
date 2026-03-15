// src/components/loading.rs
use dioxus::prelude::*;

#[component]
pub fn Loading(msg: Option<String>) -> Element {
    let message = msg.unwrap_or_else(|| "Loading…".to_string());
    rsx! {
        div { class: "loading-wrapper",
            div { class: "spinner" }
            p { class: "loading-text", "{message}" }
        }
    }
}

#[component]
pub fn ErrorBox(msg: String) -> Element {
    rsx! {
        div { class: "error-box",
            "⚠ Error: {msg}"
        }
    }
}

// ── Copy to clipboard component ────────────────────────────────────────────

#[component]
pub fn CopyButton(text: String) -> Element {
    let mut copied = use_signal(|| false);
    let text_clone = text.clone();

    rsx! {
        span { class: "copy-wrap",
            button {
                class: if *copied.read() { "copy-btn copied" } else { "copy-btn" },
                onclick: move |_: Event<MouseData>| {
                    let t = text_clone.clone();
                    let mut copied = copied.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        // Use JSON.stringify to safely encode the string for clipboard
                        let script = format!("navigator.clipboard.writeText(JSON.parse({}))", 
                            serde_json::to_string(&t).unwrap_or_default());
                        let _ = js_sys::eval(&script);
                        copied.set(true);
                        gloo_timers::future::TimeoutFuture::new(1500).await;
                        copied.set(false);
                    });
                },
                if *copied.read() { "✓ Copied" } else { "Copy" }
            }
        }
    }
}
