// src/pages/home.rs
use dioxus::prelude::*;
use crate::router::Route;
use crate::services::rpc::{
    get_latest_blocks, get_network_stats, get_avg_block_time,
    Block, NetworkStats, shorten_hash, shorten_addr, unix_to_age,
};
use crate::components::loading::{Loading, ErrorBox};
use wasm_bindgen::JsCast;

const VERSION: &str = "v0.1.4";

#[component]
pub fn HomePage() -> Element {
    let mut blocks: Signal<Vec<Block>>           = use_signal(|| vec![]);
    let mut stats:  Signal<Option<NetworkStats>> = use_signal(|| None);
    let mut loading  = use_signal(|| true);
    let mut error: Signal<Option<String>>        = use_signal(|| None);
    let mut last_updated: Signal<String>         = use_signal(|| "".to_string());
    let mut avg_block_time: Signal<f64>          = use_signal(|| 0.0);

    let do_fetch = move || {
        wasm_bindgen_futures::spawn_local(async move {
            let (stats_res, blocks_res) = futures::join!(
                get_network_stats(),
                get_latest_blocks(10)
            );
            let avg_time = get_avg_block_time(10).await;
            avg_block_time.set(avg_time);
            match stats_res {
                Ok(s)  => stats.set(Some(s)),
                Err(e) => error.set(Some(e)),
            }
            match blocks_res {
                Ok(b)  => blocks.set(b),
                Err(e) => error.set(Some(e)),
            }
            let now = js_sys::Date::new_0();
            last_updated.set(format!("{:02}:{:02}:{:02}",
                now.get_hours(), now.get_minutes(), now.get_seconds()));
            loading.set(false);
        });
    };

    use_effect(move || { do_fetch(); });

    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(30_000).await;
            do_fetch();
        }
    });

    let recent_txs: Vec<(String, u64)> = blocks.read()
        .iter()
        .flat_map(|b| b.transactions.iter().take(3).map(|h| (h.clone(), b.number)).collect::<Vec<_>>())
        .take(10)
        .collect();

    let total_txs: usize = blocks.read().iter().map(|b| b.transaction_count).sum();

    rsx! {
        div {

            // ── Hero ──────────────────────────────────────────────────
            div { class: "hero",
                div { class: "hero-inner",
                    h1 { class: "hero-title", "The Telcoin Network Explorer" }
                    p  { class: "hero-sub", "Search blocks, transactions, and addresses on the Telcoin blockchain" }
                    div { class: "search-bar",
                        input {
                            class: "search-input",
                            placeholder: "Search by Address / Txn Hash / Block Number",
                            id: "home-search",
                            onkeydown: move |e: Event<KeyboardData>| {
                                if e.key() == Key::Enter {
                                    run_search();
                                }
                            }
                        }
                        button {
                            class: "search-btn",
                            onclick: move |_: Event<MouseData>| { run_search(); },
                            "Search"
                        }
                    }
                }
            }

            // ── Stats ─────────────────────────────────────────────────
            div { class: "stats-band",
                div { class: "stats-grid-centered",
                    if let Some(s) = stats.read().as_ref() {
                        StatTile { icon: "⬡", label: "LATEST BLOCK",
                            value: format!("#{}", s.latest_block), sub: "Telcoin Network".to_string() }
                        StatTile { icon: "⛽", label: "GAS PRICE",
                            value: format!("{:.2}", s.gas_price_gwei), sub: "Gwei".to_string() }
                        StatTile { icon: "↔", label: "TRANSACTIONS",
                            value: format!("{}", total_txs), sub: "In latest 10 blocks".to_string() }
                        StatTile { icon: "🔗", label: "CHAIN ID",
                            value: format!("{}", s.chain_id), sub: "Adiri Testnet".to_string() }
                        StatTile { icon: "⏱", label: "BLOCK TIME",
                            value: {
                                let t = *avg_block_time.read();
                                if t > 0.0 { format!("{:.1}s", t) } else { "—".to_string() }
                            },
                            sub: "Avg last 10 blocks".to_string() }
                        StatTile { icon: "〜", label: "NETWORK",
                            value: "● LIVE".to_string(), sub: "rpc.telcoin.network".to_string() }
                    } else {
                        for _ in 0..6 {
                            div { class: "stat-tile stat-tile-skeleton" }
                        }
                    }
                }
            }

            // ── Panels ────────────────────────────────────────────────
            div { class: "home-content",

                div { class: "refresh-bar",
                    span { class: "refresh-dot" }
                    span { class: "refresh-text", "Auto-refreshing every 30s" }
                    if !last_updated.read().is_empty() {
                        span { class: "refresh-time",
                            " · Last updated {last_updated}"
                        }
                    }
                }

                div { class: "dual-col",

                    // Latest Blocks panel
                    div { class: "panel",
                        div { class: "panel-header",
                            span { class: "panel-icon-circ" }
                            span { class: "panel-title", "Latest Blocks" }
                        }
                        if *loading.read() {
                            Loading { msg: Some("Loading blocks…".to_string()) }
                        } else if let Some(err) = error.read().as_ref() {
                            ErrorBox { msg: err.clone() }
                        } else {
                            ul { class: "data-list",
                                for block in blocks.read().iter() {
                                    li { class: "home-row",
                                        // Block icon
                                        div { class: "home-row-icon block-row-icon" }
                                        // Block number + age
                                        div { class: "home-row-main",
                                            Link { to: Route::BlockPage { block_number: block.number },
                                                span { class: "hash-cell home-row-id",
                                                    { format!("{}", block.number) }
                                                }
                                            }
                                            span { class: "home-row-age", "{unix_to_age(block.timestamp)}" }
                                        }
                                        // Validator
                                        div { class: "home-row-mid",
                                            span { class: "home-row-label", "Validator" }
                                            Link { to: Route::AddressPage { address: block.validator.clone() },
                                                span { class: "hash-cell home-row-addr",
                                                    "{shorten_addr(&block.validator)}"
                                                }
                                            }
                                            span { class: "home-row-detail",
                                                { format!("{} txns in {} gas", block.transaction_count, block.gas_used) }
                                            }
                                        }
                                        // Tx badge
                                        div { class: "home-row-right",
                                            span { class: "tx-badge",
                                                "{block.transaction_count} txns"
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "panel-footer",
                                Link { to: Route::BlocksPage { page: 0 }, class: "panel-view-all-footer",
                                    "View All Blocks →"
                                }
                            }
                        }
                    }

                    // Latest Transactions panel
                    div { class: "panel",
                        div { class: "panel-header",
                            span { class: "panel-icon-doc" }
                            span { class: "panel-title", "Latest Transactions" }
                        }
                        if *loading.read() {
                            Loading { msg: Some("Loading transactions…".to_string()) }
                        } else if recent_txs.is_empty() {
                            div { class: "panel-empty",
                                "No transactions in the latest blocks"
                            }
                        } else {
                            ul { class: "data-list",
                                for (hash, block_num) in recent_txs.iter() {
                                    li { class: "home-row",
                                        div { class: "home-row-icon tx-row-icon" }
                                        div { class: "home-row-main",
                                            Link { to: Route::TransactionPage { hash: hash.clone() },
                                                span { class: "hash-cell home-row-id",
                                                    "{shorten_hash(hash)}"
                                                }
                                            }
                                        }
                                        div { class: "home-row-mid",
                                            span { class: "home-row-label", "Block" }
                                            Link { to: Route::BlockPage { block_number: *block_num },
                                                span { class: "hash-cell home-row-addr",
                                                    { format!("#{}", block_num) }
                                                }
                                            }
                                        }
                                        div { class: "home-row-right" }
                                    }
                                }
                            }
                        }
                    }
                }


            }
        }
    }
}

fn run_search() {
    let window = web_sys::window().unwrap();
    let doc = window.document().unwrap();
    if let Some(el) = doc.get_element_by_id("home-search") {
        let input: web_sys::HtmlInputElement = el.dyn_into().unwrap();
        let q = input.value();
        let q = q.trim();
        if q.is_empty() { return; }
        let url = if q.len() == 66 && q.starts_with("0x") {
            format!("/tx/{}", q)
        } else if q.len() == 42 && q.starts_with("0x") {
            format!("/address/{}", q)
        } else if q.chars().all(|c| c.is_ascii_digit()) {
            format!("/block/{}", q)
        } else {
            return;
        };
        window.location().set_href(&url).ok();
    }
}

#[component]
fn StatTile(icon: String, label: String, value: String, sub: String) -> Element {
    let is_live = label == "NETWORK";
    rsx! {
        div { class: "stat-tile",
            div { class: "stat-tile-label", "{label}" }
            if is_live {
                div { class: "stat-tile-value live",
                    span { class: "live-dot" }
                    "LIVE"
                }
            } else {
                div { class: "stat-tile-value", "{value}" }
            }
            div { class: "stat-tile-sub", "{sub}" }
        }
    }
}
