// src/services/rpc.rs
// Telcoin Network JSON-RPC client
// RPC: https://rpc.telcoin.network  |  Chain ID: 2017
// ConsensusRegistry: 0x07e17e17e17e17e17e17e17e17e17e17e17e17e1

use serde::{Deserialize, Serialize};
use gloo_net::http::Request;

pub const RPC_ENDPOINTS: &[&str] = &[
    "https://rpc.telcoin.network",
    "https://adiri.tel",
    "https://node1.telcoin.network",
    "https://node2.telcoin.network",
    "https://node3.telcoin.network",
    "https://node4.telcoin.network",
];

pub const CHAIN_ID: u64 = 2017;
pub const NATIVE_TOKEN: &str = "TEL";
pub const CONSENSUS_REGISTRY: &str = "0x07e17e17e17e17e17e17e17e17e17e17e17e17e1";
pub const EPOCH_DURATION_HOURS: u64 = 24;
pub const VALIDATOR_STAKE_REQUIRED: &str = "1,000,000 TEL";

// ─── JSON-RPC ─────────────────────────────────────────────────────────────

#[derive(Serialize, Debug, Clone)]
struct RpcRequest<P: Serialize + Clone> {
    jsonrpc: String,
    method:  String,
    params:  P,
    id:      u32,
}

#[derive(Deserialize, Debug)]
struct RpcResponse<T> {
    result: Option<T>,
    error:  Option<RpcError>,
}

#[derive(Deserialize, Debug)]
struct RpcError { message: String }

async fn rpc_call<P: Serialize + Clone, T: for<'de> Deserialize<'de>>(
    method: &str, params: P,
) -> Result<T, String> {
    let mut last_err = "No endpoints available".to_string();
    for endpoint in RPC_ENDPOINTS {
        let body = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: params.clone(),
            id: 1,
        };
        let body_str = serde_json::to_string(&body).map_err(|e| e.to_string())?;
        match Request::post(endpoint)
            .header("Content-Type", "application/json")
            .body(body_str).map_err(|e| e.to_string())?
            .send().await
        {
            Ok(resp) => match resp.json::<RpcResponse<T>>().await {
                Ok(r) => {
                    if let Some(e) = r.error { last_err = e.message; continue; }
                    if let Some(v) = r.result { return Ok(v); }
                }
                Err(e) => { last_err = e.to_string(); continue; }
            },
            Err(e) => { last_err = e.to_string(); continue; }
        }
    }
    Err(last_err)
}

// ─── Domain types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub number:            u64,
    pub hash:              String,
    pub parent_hash:       String,
    pub timestamp:         u64,
    pub validator:         String,
    pub gas_used:          u64,
    pub gas_limit:         u64,
    pub transaction_count: usize,
    pub transactions:      Vec<String>,
    pub size:              u64,
    pub consensus:         String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub hash:              String,
    pub from:              String,
    pub to:                Option<String>,
    pub value:             String,
    pub value_tel:         f64,
    pub gas:               u64,
    pub gas_used:          u64,
    pub gas_price:         u64,
    pub nonce:             u64,
    pub block_number:      Option<u64>,
    pub block_hash:        Option<String>,
    pub transaction_index: Option<u64>,
    pub input:             String,
    pub status:            Option<bool>,
    pub decoded_input:     Option<DecodedInput>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedInput {
    pub method:    String,
    pub signature: String,
    pub params:    Vec<(String, String)>, // (name, value)
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTransfer {
    pub tx_hash:       String,
    pub block_number:  u64,
    pub timestamp:     u64,
    pub from:          String,
    pub to:            String,
    pub token_address: String,
    pub token_symbol:  String,
    pub amount:        f64,
    pub amount_raw:    String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkStats {
    pub latest_block:   u64,
    pub gas_price_gwei: f64,
    pub chain_id:       u64,
    pub epoch_number:   Option<u64>,
}

// ─── Raw shapes ───────────────────────────────────────────────────────────

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RawBlock {
    number:       Option<String>,
    hash:         Option<String>,
    parent_hash:  Option<String>,
    timestamp:    Option<String>,
    miner:        Option<String>,
    gas_used:     Option<String>,
    gas_limit:    Option<String>,
    transactions: Option<serde_json::Value>,
    size:         Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RawTransaction {
    hash:              Option<String>,
    from:              Option<String>,
    to:                Option<String>,
    value:             Option<String>,
    gas:               Option<String>,
    gas_price:         Option<String>,
    nonce:             Option<String>,
    block_number:      Option<String>,
    block_hash:        Option<String>,
    transaction_index: Option<String>,
    input:             Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct RawReceipt {
    status:   Option<String>,
    gas_used: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RawLog {
    pub address:          Option<String>,
    pub topics:           Option<Vec<String>>,
    pub data:             Option<String>,
    #[serde(rename = "blockNumber")]
    pub block_number:     Option<String>,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<String>,
}

// ─── Helpers ──────────────────────────────────────────────────────────────

pub fn hex_to_u64(s: &str) -> u64 {
    u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).unwrap_or(0)
}

pub fn hex_wei_to_tel(s: &str) -> f64 {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.is_empty() || s == "0" { return 0.0; }
    u128::from_str_radix(s, 16).unwrap_or(0) as f64 / 1e18
}

pub fn shorten_hash(h: &str) -> String {
    if h.len() > 14 { format!("{}…{}", &h[..8], &h[h.len()-6..]) } else { h.to_string() }
}

pub fn shorten_addr(h: &str) -> String {
    if h.len() > 10 { format!("{}…{}", &h[..6], &h[h.len()-4..]) } else { h.to_string() }
}

pub fn format_tel(v: f64) -> String {
    if v == 0.0 { "0 TEL".to_string() }
    else if v < 0.0001 { format!("{:.8} TEL", v) }
    else { format!("{:.4} TEL", v) }
}

pub fn unix_to_age(ts: u64) -> String {
    let now = js_sys::Date::now() as u64 / 1000;
    let d = now.saturating_sub(ts);
    if d < 60 { format!("{} secs ago", d) }
    else if d < 3600 { format!("{} mins ago", d / 60) }
    else if d < 86400 { format!("{} hrs ago", d / 3600) }
    else { format!("{} days ago", d / 86400) }
}

pub fn unix_to_datetime(ts: u64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ts as f64 * 1000.0));
    String::from(date.to_utc_string())
}

/// Decode known EVM function selectors from input data
pub fn decode_input(input: &str) -> Option<DecodedInput> {
    if input.len() < 10 { return None; }
    let selector = &input[..10].to_lowercase();
    match selector.as_str() {
        // ERC-20
        "0xa9059cbb" => Some(DecodedInput {
            method: "transfer".to_string(),
            signature: "transfer(address,uint256)".to_string(),
            params: vec![
                ("to".to_string(),     extract_address(&input[10..10+64])),
                ("amount".to_string(), extract_uint256(&input[74..74+64])),
            ],
        }),
        "0x23b872dd" => Some(DecodedInput {
            method: "transferFrom".to_string(),
            signature: "transferFrom(address,address,uint256)".to_string(),
            params: vec![
                ("from".to_string(),   extract_address(&input[10..10+64])),
                ("to".to_string(),     extract_address(&input[74..74+64])),
                ("amount".to_string(), extract_uint256(&input[138..138+64])),
            ],
        }),
        "0x095ea7b3" => Some(DecodedInput {
            method: "approve".to_string(),
            signature: "approve(address,uint256)".to_string(),
            params: vec![
                ("spender".to_string(), extract_address(&input[10..10+64])),
                ("amount".to_string(),  extract_uint256(&input[74..74+64])),
            ],
        }),
        // ConsensusRegistry
        "0x4e71d92d" => Some(DecodedInput {
            method: "claim".to_string(),
            signature: "claim()".to_string(),
            params: vec![],
        }),
        "0x3ccfd60b" => Some(DecodedInput {
            method: "withdraw".to_string(),
            signature: "withdraw()".to_string(),
            params: vec![],
        }),
        "0xd0e30db0" => Some(DecodedInput {
            method: "deposit".to_string(),
            signature: "deposit()".to_string(),
            params: vec![],
        }),
        "0x60806040" => Some(DecodedInput {
            method: "Contract Deployment".to_string(),
            signature: "constructor(...)".to_string(),
            params: vec![],
        }),
        _ => Some(DecodedInput {
            method: format!("Unknown ({})", selector),
            signature: selector.to_string(),
            params: vec![],
        }),
    }
}

fn extract_address(hex: &str) -> String {
    if hex.len() >= 64 {
        format!("0x{}", &hex[24..64])
    } else {
        format!("0x{}", hex)
    }
}

fn extract_uint256(hex: &str) -> String {
    if hex.is_empty() { return "0".to_string(); }
    let trimmed = hex.trim_start_matches('0');
    if trimmed.is_empty() { return "0".to_string(); }
    match u128::from_str_radix(trimmed, 16) {
        Ok(v) => v.to_string(),
        Err(_) => format!("0x{}", hex),
    }
}

// ─── Network ──────────────────────────────────────────────────────────────

pub async fn get_block_number() -> Result<u64, String> {
    let r: String = rpc_call("eth_blockNumber", serde_json::json!([])).await?;
    Ok(hex_to_u64(&r))
}

pub async fn get_gas_price() -> Result<f64, String> {
    let r: String = rpc_call("eth_gasPrice", serde_json::json!([])).await?;
    Ok(hex_to_u64(&r) as f64 / 1e9)
}

pub async fn get_epoch_info() -> Option<u64> {
    let call = serde_json::json!([{"to": CONSENSUS_REGISTRY, "data": "0x45c8b1a6"}, "latest"]);
    match rpc_call::<_, String>("eth_call", call).await {
        Ok(hex) => { let e = hex_to_u64(&hex); if e > 0 { Some(e) } else { None } }
        Err(_)  => None,
    }
}

pub async fn get_network_stats() -> Result<NetworkStats, String> {
    let latest_block   = get_block_number().await?;
    let gas_price_gwei = get_gas_price().await.unwrap_or(0.0);
    let epoch_number   = get_epoch_info().await;
    Ok(NetworkStats { latest_block, gas_price_gwei, chain_id: CHAIN_ID, epoch_number })
}

// ─── Blocks ───────────────────────────────────────────────────────────────

pub async fn get_block_by_number(n: u64) -> Result<Block, String> {
    let raw: RawBlock = rpc_call("eth_getBlockByNumber",
        serde_json::json!([format!("0x{:x}", n), false])).await?;
    let txs: Vec<String> = match &raw.transactions {
        Some(serde_json::Value::Array(a)) =>
            a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
        _ => vec![],
    };
    Ok(Block {
        number:            raw.number.as_deref().map(hex_to_u64).unwrap_or(n),
        hash:              raw.hash.unwrap_or_default(),
        parent_hash:       raw.parent_hash.unwrap_or_default(),
        timestamp:         raw.timestamp.as_deref().map(hex_to_u64).unwrap_or(0),
        validator:         raw.miner.unwrap_or_default(),
        gas_used:          raw.gas_used.as_deref().map(hex_to_u64).unwrap_or(0),
        gas_limit:         raw.gas_limit.as_deref().map(hex_to_u64).unwrap_or(0),
        transaction_count: txs.len(),
        transactions:      txs,
        size:              raw.size.as_deref().map(hex_to_u64).unwrap_or(0),
        consensus:         "DAG (Narwhal/Bullshark)".to_string(),
    })
}

pub async fn get_latest_blocks(count: u64) -> Result<Vec<Block>, String> {
    let latest = get_block_number().await?;
    let mut blocks = Vec::new();
    for i in 0..count {
        if let Ok(b) = get_block_by_number(latest.saturating_sub(i)).await {
            blocks.push(b);
        }
    }
    Ok(blocks)
}

/// Fetch a page of blocks (for the block list page)
pub async fn get_blocks_page(page: u64, per_page: u64) -> Result<(Vec<Block>, u64), String> {
    let latest = get_block_number().await?;
    let start  = latest.saturating_sub(page * per_page);
    let mut blocks = Vec::new();
    for i in 0..per_page {
        if start < i { break; }
        if let Ok(b) = get_block_by_number(start.saturating_sub(i)).await {
            blocks.push(b);
        }
    }
    Ok((blocks, latest))
}

// ─── Transactions ─────────────────────────────────────────────────────────

pub async fn get_transaction(hash: &str) -> Result<Transaction, String> {
    let raw: RawTransaction =
        rpc_call("eth_getTransactionByHash", serde_json::json!([hash])).await?;
    let receipt: Option<RawReceipt> =
        rpc_call("eth_getTransactionReceipt", serde_json::json!([hash])).await.ok();
    let status   = receipt.as_ref().and_then(|r| r.status.as_deref()).map(|s| s == "0x1");
    let gas_used = receipt.as_ref().and_then(|r| r.gas_used.as_deref()).map(hex_to_u64).unwrap_or(0);
    let value_hex = raw.value.clone().unwrap_or_else(|| "0x0".to_string());
    let input = raw.input.clone().unwrap_or_default();
    let decoded_input = if input.len() >= 10 { decode_input(&input) } else { None };
    Ok(Transaction {
        hash:              raw.hash.unwrap_or_default(),
        from:              raw.from.unwrap_or_default(),
        to:                raw.to,
        value:             value_hex.clone(),
        value_tel:         hex_wei_to_tel(&value_hex),
        gas:               raw.gas.as_deref().map(hex_to_u64).unwrap_or(0),
        gas_used,
        gas_price:         raw.gas_price.as_deref().map(hex_to_u64).unwrap_or(0),
        nonce:             raw.nonce.as_deref().map(hex_to_u64).unwrap_or(0),
        block_number:      raw.block_number.as_deref().map(hex_to_u64),
        block_hash:        raw.block_hash,
        transaction_index: raw.transaction_index.as_deref().map(hex_to_u64),
        input,
        status,
        decoded_input,
    })
}

// ─── Address ──────────────────────────────────────────────────────────────

pub async fn get_balance(address: &str) -> Result<f64, String> {
    let r: String = rpc_call("eth_getBalance", serde_json::json!([address, "latest"])).await?;
    Ok(hex_wei_to_tel(&r))
}

pub async fn get_tx_count(address: &str) -> Result<u64, String> {
    let r: String = rpc_call("eth_getTransactionCount", serde_json::json!([address, "latest"])).await?;
    Ok(hex_to_u64(&r))
}

pub async fn get_token_transfers(address: &str, from_block: u64, to_block: u64) -> Result<Vec<RawLog>, String> {
    let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
    let params = serde_json::json!([{
        "fromBlock": format!("0x{:x}", from_block),
        "toBlock":   format!("0x{:x}", to_block),
        "topics":    [transfer_topic, null, null],
        "address":   null
    }]);
    let logs: Vec<RawLog> = rpc_call("eth_getLogs", params).await.unwrap_or_default();
    let lower  = address.to_lowercase();
    let suffix = lower.strip_prefix("0x").unwrap_or(&lower);
    Ok(logs.into_iter().filter(|l| {
        l.topics.as_ref().map(|t| {
            t.get(1).map(|s| s.ends_with(suffix)).unwrap_or(false) ||
            t.get(2).map(|s| s.ends_with(suffix)).unwrap_or(false)
        }).unwrap_or(false)
    }).collect())
}

pub fn parse_transfer_logs(logs: Vec<RawLog>) -> Vec<TokenTransfer> {
    logs.into_iter().filter_map(|log| {
        let topics = log.topics.as_ref()?;
        if topics.len() < 3 { return None; }
        let from = format!("0x{}", &topics[1][topics[1].len().saturating_sub(40)..]);
        let to   = format!("0x{}", &topics[2][topics[2].len().saturating_sub(40)..]);
        let raw  = log.data.clone().unwrap_or_else(|| "0x0".to_string());
        Some(TokenTransfer {
            tx_hash:       log.transaction_hash.unwrap_or_default(),
            block_number:  log.block_number.as_deref().map(hex_to_u64).unwrap_or(0),
            timestamp:     0,
            from, to,
            token_address: log.address.clone().unwrap_or_default(),
            token_symbol:  "ERC-20".to_string(),
            amount:        hex_wei_to_tel(&raw),
            amount_raw:    raw,
        })
    }).collect()
}

// ─── Validators ───────────────────────────────────────────────────────────

pub async fn get_recent_validators(block_count: u64) -> Result<Vec<String>, String> {
    let latest = get_block_number().await?;
    let mut seen = std::collections::BTreeSet::new();
    for i in 0..block_count.min(50) {
        if let Ok(b) = get_block_by_number(latest.saturating_sub(i)).await {
            if !b.validator.is_empty() &&
               b.validator != "0x0000000000000000000000000000000000000000" {
                seen.insert(b.validator);
            }
        }
    }
    Ok(seen.into_iter().collect())
}

// ─── Contract Detection ───────────────────────────────────────────────────

pub async fn is_contract(address: &str) -> bool {
    let params = serde_json::json!([address, "latest"]);
    let code: String = rpc_call("eth_getCode", params).await.unwrap_or_default();
    // "0x" or "0x0" means EOA, anything longer is a contract
    code.len() > 3
}

// ─── Validators from ConsensusRegistry ───────────────────────────────────

pub async fn get_validators_from_registry() -> Result<Vec<String>, String> {
    // Call getValidators() selector: 0xb7ab4db5
    let params = serde_json::json!([{
        "to": "0x07e17e17e17e17e17e17e17e17e17e17e17e17e1",
        "data": "0xb7ab4db5"
    }, "latest"]);
    let result: String = rpc_call("eth_call", params).await.unwrap_or_default();
    if result.len() < 10 {
        // Fallback to scanning recent blocks
        return get_recent_validators(50).await;
    }
    // Parse ABI-encoded address array
    // Skip first 32 bytes (offset) + 32 bytes (length)
    let hex = result.strip_prefix("0x").unwrap_or(&result);
    if hex.len() < 128 {
        return get_recent_validators(50).await;
    }
    let count_hex = &hex[64..128];
    let count = u64::from_str_radix(count_hex, 16).unwrap_or(0) as usize;
    let mut validators = Vec::new();
    for i in 0..count {
        let start = 128 + i * 64;
        let end   = start + 64;
        if end > hex.len() { break; }
        let addr_hex = &hex[start..end];
        let addr = format!("0x{}", &addr_hex[24..]); // last 20 bytes
        validators.push(addr);
    }
    if validators.is_empty() {
        return get_recent_validators(50).await;
    }
    Ok(validators)
}
