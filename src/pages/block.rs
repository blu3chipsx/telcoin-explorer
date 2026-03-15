// src/pages/block.rs
use dioxus::prelude::*;
use crate::router::Route;
use crate::services::rpc::{get_block_by_number, Block, shorten_hash, shorten_addr, unix_to_age, unix_to_datetime};
use crate::components::loading::{Loading, ErrorBox, CopyButton};

#[component]
pub fn BlockPage(block_number: u64) -> Element {
    let block: Signal<Option<Block>> = use_signal(|| None);
    let loading = use_signal(|| true);
    let error: Signal<Option<String>> = use_signal(|| None);

    use_effect(move || {
        let mut block   = block.clone();
        let mut loading = loading.clone();
        let mut error   = error.clone();
        wasm_bindgen_futures::spawn_local(async move {
            loading.set(true);
            match get_block_by_number(block_number).await {
                Ok(b)  => block.set(Some(b)),
                Err(e) => error.set(Some(e)),
            }
            loading.set(false);
        });
    });

    rsx! {
        div { class: "page",
            h1 { class: "page-title",
                "Block " span { "#{block_number}" }
            }

            if *loading.read() {
                Loading { msg: Some(format!("Fetching block #{}…", block_number)) }
            } else if let Some(err) = error.read().as_ref() {
                ErrorBox { msg: err.clone() }
            } else if let Some(b) = block.read().as_ref() {
                div { class: "detail-grid",
                    div { class: "detail-panel",
                        div { class: "detail-panel-title", "Block Overview" }
                        div { class: "detail-table",

                            div { class: "detail-row",
                                div { class: "detail-key", "Block Height" }
                                div { class: "detail-val", "#{b.number}" }
                            }
                            div { class: "detail-row",
                                div { class: "detail-key", "Block Hash" }
                                div { class: "detail-val", "{b.hash}" }
                            }
                            div { class: "detail-row",
                                div { class: "detail-key", "Parent Hash" }
                                div { class: "detail-val",
                                    if b.number > 0 {
                                        Link { to: Route::BlockPage { block_number: b.number - 1 },
                                            span { class: "hash-cell", "{b.parent_hash}" }
                                        }
                                    } else {
                                        span { "{b.parent_hash}" }
                                    }
                                }
                            }
                            div { class: "detail-row",
                                div { class: "detail-key", "Timestamp" }
                                div { class: "detail-val",
                                    { format!("{}  ({})", unix_to_datetime(b.timestamp), unix_to_age(b.timestamp)) }
                                }
                            }
                            // Renamed: Miner → Validator
                            div { class: "detail-row",
                                div { class: "detail-key", "Validator" }
                                div { class: "detail-val",
                                    Link { to: Route::AddressPage { address: b.validator.clone() },
                                        span { class: "hash-cell", "{b.validator}" }
                                    }
                                    span { class: "chip success", style: "margin-left:10px; font-size:10px;", "CVV" }
                                }
                            }
                            div { class: "detail-row",
                                div { class: "detail-key", "Consensus" }
                                div { class: "detail-val",
                                    span { class: "chip pending", "{b.consensus}" }
                                }
                            }
                            div { class: "detail-row",
                                div { class: "detail-key", "Finality" }
                                div { class: "detail-val",
                                    span { class: "chip success", "✓ Instant (DAG)" }
                                }
                            }
                            div { class: "detail-row",
                                div { class: "detail-key", "Transactions" }
                                div { class: "detail-val", "{b.transaction_count} transactions" }
                            }
                            div { class: "detail-row",
                                div { class: "detail-key", "Gas Used" }
                                div { class: "detail-val",
                                    {
                                        let pct = if b.gas_limit > 0 { b.gas_used as f64 / b.gas_limit as f64 * 100.0 } else { 0.0 };
                                        format!("{}  /  {}  ({:.1}%)", b.gas_used, b.gas_limit, pct)
                                    }
                                }
                            }
                            div { class: "detail-row",
                                div { class: "detail-key", "Size" }
                                div { class: "detail-val", "{b.size} bytes" }
                            }
                        }
                    }

                    if !b.transactions.is_empty() {
                        div { class: "detail-panel",
                            div { class: "detail-panel-title", "{b.transaction_count} Transactions" }
                            ul { class: "data-list",
                                for tx_hash in b.transactions.iter() {
                                    li { class: "data-row",
                                        div { class: "row-icon tx",
                                            svg { xmlns: "http://www.w3.org/2000/svg", width: "15", height: "15", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.5",
                                                path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
                                                polyline { points: "14 2 14 8 20 8" }
                                            }
                                        }
                                        div { class: "row-main",
                                            Link { to: Route::TransactionPage { hash: tx_hash.clone() },
                                                span { class: "row-id", "{shorten_hash(tx_hash)}" }
                                            }
                                            div { class: "row-meta", "Block #{b.number}" }
                                        }
                                        div { class: "row-side",
                                            span { class: "row-badge tx", "TX" }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "detail-panel",
                            div { class: "empty-state", "No transactions in this block." }
                        }
                    }

                    div { style: "display: flex; gap: 12px; align-items: center;",
                        if b.number > 0 {
                            Link { to: Route::BlockPage { block_number: b.number - 1 },
                                button { class: "page-btn", "← Previous Block" }
                            }
                        }
                        Link { to: Route::BlockPage { block_number: b.number + 1 },
                            button { class: "page-btn", "Next Block →" }
                        }
                    }
                }
            }
        }
    }
}
