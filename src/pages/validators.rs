// src/pages/validators.rs
use dioxus::prelude::*;
use crate::router::Route;
use crate::services::rpc::{
    get_validators_from_registry, get_block_number, get_epoch_info,
    shorten_addr, CONSENSUS_REGISTRY, VALIDATOR_STAKE_REQUIRED,
    EPOCH_DURATION_HOURS,
};
use crate::components::loading::{Loading, ErrorBox};

#[component]
pub fn ValidatorsPage() -> Element {
    let validators: Signal<Vec<String>>      = use_signal(|| vec![]);
    let epoch_num:  Signal<Option<u64>>      = use_signal(|| None);
    let loading     = use_signal(|| true);
    let error: Signal<Option<String>>        = use_signal(|| None);

    use_effect(move || {
        let mut validators = validators.clone();
        let mut epoch_num  = epoch_num.clone();
        let mut loading    = loading.clone();
        let mut error      = error.clone();
        wasm_bindgen_futures::spawn_local(async move {
            loading.set(true);
            // Fetch epoch number
            epoch_num.set(get_epoch_info().await);
            // Fetch validators from recent blocks
            match get_validators_from_registry().await {
                Ok(v)  => validators.set(v),
                Err(e) => error.set(Some(e)),
            }
            loading.set(false);
        });
    });

    // Compute epoch sub-text before rsx
    let epoch_sub = {
        let ep = *epoch_num.read();
        match ep {
            Some(e) => format!("Current epoch: {}", e),
            None    => "Rotating validator committees".to_string(),
        }
    };

    rsx! {
        div { class: "page",

            div { class: "page-title-row",
                h1 { class: "page-title", "Validators" }
                a {
                    href: "https://tnips.telcoin.network/tnips/tnips/tnip_2.html",
                    target: "_blank",
                    class: "docs-link",
                    "TNIP-2 Spec ↗"
                }
            }

            // Info cards
            div { class: "validator-info-grid",
                div { class: "info-card",
                    div { class: "info-card-icon", "⬡" }
                    div {
                        div { class: "info-card-label", "Consensus Type" }
                        div { class: "info-card-value", "DAG — Narwhal/Bullshark" }
                        div { class: "info-card-sub", "Parallel batch collection with causal ordering" }
                    }
                }
                div { class: "info-card",
                    div { class: "info-card-icon", "⏱" }
                    div {
                        div { class: "info-card-label", "Epoch Duration" }
                        div { class: "info-card-value", "{EPOCH_DURATION_HOURS} Hours" }
                        div { class: "info-card-sub", "{epoch_sub}" }
                    }
                }
                div { class: "info-card",
                    div { class: "info-card-icon", "🔒" }
                    div {
                        div { class: "info-card-label", "Required Stake" }
                        div { class: "info-card-value", "{VALIDATOR_STAKE_REQUIRED}" }
                        div { class: "info-card-sub", "GSMA MNO validators only" }
                    }
                }
                div { class: "info-card",
                    div { class: "info-card-icon", "📋" }
                    div {
                        div { class: "info-card-label", "Registry Contract" }
                        div { class: "info-card-value mono-sm",
                            Link { to: Route::AddressPage { address: CONSENSUS_REGISTRY.to_string() },
                                span { class: "hash-cell", "{shorten_addr(CONSENSUS_REGISTRY)}" }
                            }
                        }
                        div { class: "info-card-sub", "ConsensusRegistry @ 0x07e1…" }
                    }
                }
            }

            // Validator types
            div { class: "panel", style: "margin-bottom: 20px;",
                div { class: "panel-header",
                    div { class: "panel-title",
                        div { class: "panel-title-icon block-icon", "?" }
                        "Validator Types"
                    }
                }
                div { class: "validator-types-grid",
                    div { class: "vtype-card",
                        div { class: "vtype-badge cvv", "CVV" }
                        div { class: "vtype-name", "Committee Voting Validators" }
                        div { class: "vtype-desc",
                            "Currently active validators casting votes, extending the canonical chain, and reaching consensus on transactions."
                        }
                    }
                    div { class: "vtype-card",
                        div { class: "vtype-badge nvv", "NVV" }
                        div { class: "vtype-name", "Non-Voting Validators" }
                        div { class: "vtype-desc",
                            "Track and execute consensus but do not vote every round. Vote on execution results at epoch transitions."
                        }
                    }
                    div { class: "vtype-card",
                        div { class: "vtype-badge ov", "OV" }
                        div { class: "vtype-name", "Observing Validators" }
                        div { class: "vtype-desc",
                            "Track and execute consensus but never vote. Used as independent verification clients."
                        }
                    }
                }
            }

            // Active validators list
            div { class: "panel",
                div { class: "panel-header",
                    div { class: "panel-title",
                        div { class: "panel-title-icon block-icon",
                            svg { xmlns: "http://www.w3.org/2000/svg", width: "14", height: "14", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                path { d: "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" }
                                circle { cx: "9", cy: "7", r: "4" }
                                path { d: "M23 21v-2a4 4 0 0 0-3-3.87" }
                                path { d: "M16 3.13a4 4 0 0 1 0 7.75" }
                            }
                        }
                        "Active Validators"
                        span { class: "panel-count",
                            "({validators.read().len()} seen in last 50 blocks)"
                        }
                    }
                }

                if *loading.read() {
                    Loading { msg: Some("Scanning recent blocks for validators…".to_string()) }
                } else if let Some(err) = error.read().as_ref() {
                    ErrorBox { msg: err.clone() }
                } else if validators.read().is_empty() {
                    div { class: "empty-state",
                        "No validator addresses found in recent blocks."
                    }
                } else {
                    div { class: "table-wrapper",
                        table { class: "tx-table",
                            thead {
                                tr {
                                    th { "#" }
                                    th { "VALIDATOR ADDRESS" }
                                    th { "ROLE" }
                                    th { "STATUS" }
                                    th { "REQUIRED STAKE" }
                                    th { "ACTIONS" }
                                }
                            }
                            tbody {
                                for (i, addr) in validators.read().iter().enumerate() {
                                    tr {
                                        td { style: "color: var(--text-muted);", "{i + 1}" }
                                        td {
                                            Link { to: Route::AddressPage { address: addr.clone() },
                                                span { class: "hash-cell", "{addr}" }
                                            }
                                        }
                                        td { span { class: "vtype-badge cvv", "CVV" } }
                                        td { span { class: "chip success", "Active" } }
                                        td { style: "color: var(--text-secondary);", "1,000,000 TEL" }
                                        td {
                                            Link { to: Route::AddressPage { address: addr.clone() },
                                                span { class: "action-link", "View →" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Info note
            div { class: "info-note",
                span { class: "info-note-icon", "ℹ" }
                div {
                    strong { "About Epoch Committees: " }
                    "Every 24 hours, the ConsensusRegistry at "
                    code { "{CONSENSUS_REGISTRY}" }
                    " runs concludeEpoch() to rotate the validator committee using Fisher-Yates shuffle seeded by the epoch's aggregate BLS12-381 signature. "
                    "Validators must stake 1M TEL and hold a GSMA ConsensusNFT. "
                    a { href: "https://tnips.telcoin.network/tnips/tnips/tnip_2.html", target: "_blank", "Read TNIP-2 ↗" }
                }
            }
        }
    }
}
