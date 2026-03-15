# ⬡ TelScan — Telcoin Network Block Explorer

A full-featured blockchain explorer for the **Telcoin Network (Adiri Testnet)** built with [Dioxus](https://dioxuslabs.com/) — the Rust framework for building web apps compiled to WebAssembly.

---

## 🌐 Network Details

| Property       | Value                          |
|----------------|-------------------------------|
| Network        | Telcoin Network (Adiri)       |
| Chain ID       | `2017`                        |
| RPC Endpoint   | `https://rpc.telcoin.network` |
| Native Token   | `TEL`                         |
| EVM Compatible | ✅ Yes                        |

---

## ✨ Features

- **Latest Blocks** — Live feed of the most recent blocks with tx counts, miner, gas usage
- **Transaction Search** — Look up any transaction by hash, see status, value, from/to, input data
- **Address Lookup** — View TEL balance, nonce, and full ERC-20 transfer history
- **Token Transfers** — Decodes `Transfer(address,address,uint256)` ERC-20 events via `eth_getLogs`
- **Smart Search Bar** — Auto-routes based on input (block number / tx hash / address)
- **Live Status Bar** — Polls network every 12s for latest block + gas price
- **Client-side Routing** — Full SPA with deep-linkable URLs

---

## 🛠 Prerequisites

You need the following installed:

### 1. Rust + Cargo
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. Add the WASM target
```bash
rustup target add wasm32-unknown-unknown
```

### 3. Install Dioxus CLI
```bash
cargo install dioxus-cli
```
> This installs the `dx` command. It may take a few minutes to compile.

### 4. Verify everything is working
```bash
dx --version    # should print: dioxus 0.6.x
cargo --version # should print: cargo 1.xx.x
```

---

## 🚀 Running the Project

### Development (with hot reload)
```bash
cd telcoin-explorer
dx serve
```
Then open **http://localhost:8080** in your browser.

The `dx serve` command will:
- Compile your Rust code to WebAssembly
- Start a local dev server
- **Hot-reload** the app whenever you save a file

> ⏱ First compile takes 1–3 minutes as it downloads all dependencies. Subsequent reloads are fast.

### Production Build
```bash
dx build --release
```
Output goes to `dist/`. You can deploy this folder to any static host (Netlify, Vercel, Cloudflare Pages, GitHub Pages, etc.).

---

## 📁 Project Structure

```
telcoin-explorer/
├── Cargo.toml              # Rust dependencies
├── Dioxus.toml             # Dioxus build config
├── assets/
│   └── main.css            # All styles (retro terminal theme)
└── src/
    ├── main.rs             # App entry point + root component
    ├── router.rs           # URL routes definition
    ├── components/
    │   ├── mod.rs
    │   ├── header.rs       # Top nav + search bar
    │   ├── status_bar.rs   # Live network stats strip
    │   └── loading.rs      # Loading spinner + error box
    ├── pages/
    │   ├── mod.rs
    │   ├── home.rs         # Dashboard: latest blocks + txs
    │   ├── block.rs        # Block detail page
    │   ├── transaction.rs  # Transaction detail page
    │   ├── address.rs      # Address: balance + ERC-20 transfers
    │   └── not_found.rs    # 404 page
    └── services/
        ├── mod.rs
        └── rpc.rs          # Telcoin JSON-RPC client + data types
```

---

## 🔌 How the RPC Client Works

All blockchain data is fetched in `src/services/rpc.rs` using standard **Ethereum JSON-RPC** calls over HTTP (Telcoin Network is EVM-compatible):

| Function                  | RPC Method                    | Used For                        |
|---------------------------|-------------------------------|----------------------------------|
| `get_block_number()`      | `eth_blockNumber`             | Latest block + status bar        |
| `get_block_by_number()`   | `eth_getBlockByNumber`        | Block detail page                |
| `get_latest_blocks(n)`    | `eth_getBlockByNumber` × n    | Home page block list             |
| `get_transaction()`       | `eth_getTransactionByHash`    | Transaction detail               |
|                           | `eth_getTransactionReceipt`   | Transaction success/fail status  |
| `get_balance()`           | `eth_getBalance`              | Address TEL balance              |
| `get_tx_count()`          | `eth_getTransactionCount`     | Address nonce                    |
| `get_token_transfers()`   | `eth_getLogs`                 | ERC-20 Transfer events           |
| `get_gas_price()`         | `eth_gasPrice`                | Status bar gas display           |

---

## 🔍 Search Routing Logic

The search bar in `src/components/header.rs` auto-routes based on your input:

| Input Format            | Routed To          |
|-------------------------|--------------------|
| `0x` + 64 hex chars     | Transaction page   |
| `0x` + 40 hex chars     | Address page       |
| All digits              | Block number page  |
| Anything else           | Transaction page   |

---

## 🎨 Design

The UI uses a **retro terminal / green-on-black CRT aesthetic** with:
- **JetBrains Mono** monospace font for data
- **Syne** display font for headings
- Amber + green on near-black palette
- CSS scanline overlay for atmosphere
- Dot-grid background
- Animated hex logo + pulsing network status dot
- Zero external UI framework dependencies — pure CSS

---

## 🧩 Extending the Explorer

### Add a new page
1. Create `src/pages/mypage.rs` with a `#[component] pub fn MyPage() -> Element`
2. Add `pub mod mypage;` to `src/pages/mod.rs`
3. Add a route variant to `src/router.rs`:
   ```rust
   #[route("/mypath/:param")]
   MyPage { param: String },
   ```
4. Link to it anywhere with:
   ```rust
   Link { to: Route::MyPage { param: "value".to_string() }, ... }
   ```

### Add a new RPC call
Add a new `async fn` in `src/services/rpc.rs` following the existing patterns:
```rust
pub async fn get_something(param: &str) -> Result<MyType, String> {
    let raw: RawType = rpc_call("eth_someMethod", serde_json::json!([param])).await?;
    // transform raw -> MyType
    Ok(...)
}
```

### Add token name/symbol resolution
In `src/services/rpc.rs`, implement `get_token_info(contract_address)` using `eth_call` with the ERC-20 `name()` and `symbol()` function selectors:
- `name()` selector: `0x06fdde03`
- `symbol()` selector: `0x95d89b41`

---

## 🐛 Troubleshooting

**`dx` command not found**
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

**WASM target missing**
```bash
rustup target add wasm32-unknown-unknown
```

**Compile errors about `js-sys` or `web-sys`**
These are WASM-only crates. Make sure you are building with `dx serve` (not `cargo run`).

**CORS errors in browser console**
Telcoin's RPC endpoint allows cross-origin requests. If you see CORS errors, check your browser extensions or try a different browser.

**Blank page after `dx serve`**
Check the browser console (F12) for errors. The most common cause is a panic in WASM — the error message will tell you exactly where.

---

## 📦 Key Dependencies

| Crate              | Purpose                              |
|--------------------|--------------------------------------|
| `dioxus`           | UI framework (React-like, Rust)      |
| `dioxus-router`    | Client-side SPA routing              |
| `gloo-net`         | HTTP requests from WASM              |
| `gloo-timers`      | Async timers (polling)               |
| `serde` / `serde_json` | JSON serialization               |
| `wasm-bindgen`     | Rust ↔ JavaScript bridge            |
| `js-sys`           | JS standard library access (Date)    |
| `wasm-logger`      | Logging to browser console           |

---

## 📄 License

MIT — build whatever you want with this.
