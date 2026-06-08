# Contributing to ZeroClaw ICP

ZeroClaw ICP is built in the open. Contributions welcome.

## Quick Start

```bash
git clone https://github.com/YOUR_USERNAME/zeroclaw-icp
cd zeroclaw-icp
cargo build --release --manifest-path canisters/vault/Cargo.toml --target wasm32-unknown-unknown
cargo build --release --manifest-path canisters/agent/Cargo.toml --target wasm32-unknown-unknown
dfx start --background
dfx deploy vault --network local
dfx deploy agent --network local
```

## What to Contribute

### High Value
- **New tools** — implement the Tool trait in `canisters/agent/src/tools/`
- **New providers** — add a new backend to the Provider enum
- **Cycles optimization** — tighter `max_response_bytes`, smarter routing
- **Tests** — pocket-ic integration tests for the agent loop

### Medium Value
- **Frontend improvements** — better UX, additional components
- **Documentation** — cost guides, setup walkthroughs
- **Example SOPs** — pre-built task templates

### Lower Priority
- **New agent behaviors** — reflex layer, memory compression
- **Multi-agent coordination** — inter-agent messaging protocol

## Code Style

- Rust: run `cargo fmt` before committing
- TypeScript: run `npm run build` to check for type errors
- No `unwrap()` in canister code — use `?` or explicit error handling
- All public canister methods must be documented in the Candid interface

## Canister Development

### Adding a new tool

1. Create `canisters/agent/src/tools/your_tool.rs`
2. Implement `pub fn execute(args: &[String]) -> ToolResult`
3. Add `pub mod your_tool;` to `tools/mod.rs`
4. Export with `pub use your_tool::execute;`
5. Add a canister method in `lib.rs` that calls it

### Adding a new provider

1. Add variant to `Provider` enum in `provider.rs`
2. Implement `complete()` method for your backend
3. Update `select_provider_for_task()` if needed

## Pull Request Checklist

- [ ] `cargo build --release` succeeds for both canisters
- [ ] `dfx deploy` succeeds on local network
- [ ] New public methods have Candid interface entries
- [ ] No `unwrap()` in persistent code paths
- [ ] Tests added for new functionality

## Architecture Philosophy

ZeroClaw ICP is designed around three principles:

1. **Trait-based**: Provider, Memory, Tool — swap implementations freely
2. **Single-threaded**: No tokio multi-threading, ICP cooperative async only
3. **Survives upgrades**: All state in stable memory, timers re-registered in post_upgrade

## Getting Help

- Open an issue for bugs or feature requests
- Discussions tab for architecture questions
- See docs/architecture.md for design decisions

---

*This project shares zero code with zeroclaw-labs/zeroclaw. We share philosophy and trait boundaries only.*
