# webpg

A Rust web playground built with Salvo, Askama, SQLite, and WebAssembly.

Three demo pages: a WASM-powered counter, a server-rendered todo list, and a real-time chat over WebTransport.

## Project structure

```
webpg/
├── server/          Salvo web server (HTML templates, routes, database, WebTransport)
├── counter/         Rust library compiled to WebAssembly (counter logic)
├── chat/            Rust library compiled to WebAssembly (WebTransport chat client)
└── .data/           Runtime artifacts — database and TLS certificates (gitignored)
```

**Why a Cargo workspace?** The server compiles to a native binary, while the WASM crates compile to `wasm32-unknown-unknown`. A workspace lets them share a lockfile and dependency versions without coupling their build targets.

**Why separate WASM crates?** Each WASM module is a self-contained library with its own `wasm-bindgen` surface. Keeping them as individual crates means they compile independently and produce focused, minimal `.wasm` binaries.

**Why `.cargo/config.toml`?** The `chat` crate uses WebTransport, which is behind an unstable feature gate in `web-sys`. The config sets `--cfg=web_sys_unstable_apis` globally so all crates compile with a plain `cargo build` and the IDE (rust-analyzer) resolves types correctly.

**Why `.data/`?** Certificates and the SQLite database are runtime artifacts that should not be committed. Putting them in a single gitignored directory keeps the project root clean and makes cleanup trivial (`rm -rf .data/`).

## Prerequisites

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

## Quick start

Build WASM crates and generate browser bindings:

```bash
cargo build -p counter -p chat --target wasm32-unknown-unknown

wasm-bindgen target/wasm32-unknown-unknown/debug/counter.wasm \
  --out-dir server/assets/wasm --target web --out-name counter

wasm-bindgen target/wasm32-unknown-unknown/debug/chat.wasm \
  --out-dir server/assets/wasm --target web --out-name chat
```

Run the server:

```bash
cargo run
```

Open https://localhost:3000/

The server generates a self-signed TLS certificate on first run and saves it to `.data/`. Subsequent restarts reuse the same certificate so the WebTransport fingerprint stays stable. Delete `.data/cert.pem` and `.data/key.pem` to regenerate.

## Development

### After editing `counter/` or `chat/`

Rebuild the changed crate and regenerate its bindings:

```bash
cargo build -p counter --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/counter.wasm \
  --out-dir server/assets/wasm --target web --out-name counter
```

### After editing `server/`

Just restart:

```bash
cargo run
```

### Tests

```bash
cargo test
```

Runs all workspace tests — server routes/templates and counter unit tests. The chat crate has no unit tests (it targets browser APIs only).

## Database

SQLite database at `.data/webpg.db`, created automatically on first run.
