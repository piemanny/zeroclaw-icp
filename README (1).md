# 🦀 ZeroClaw ICP

**Persistent. Autonomous. Self-funding. Decentralized.**  
Deploy AI agents on the Internet Computer that run forever — no server, no VPS, no maintenance.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache%202.0-blue.svg)](#license)
[![ICP](https://img.shields.io/badge/platform-Internet%20Computer-blue)](https://internetcomputer.org)
[![Rust](https://img.shields.io/badge/language-Rust-orange)](https://www.rust-lang.org)
[![Built with ic-cdk](https://img.shields.io/badge/built%20with-ic--cdk-purple)](https://crates.io/crates/ic-cdk)
[![Status: Alpha](https://img.shields.io/badge/status-alpha-red)](https://github.com/YOUR_USERNAME/zeroclaw-icp/releases)

---

> **ZeroClaw ICP** is a framework for deploying the ZeroClaw agent model onto the 
> Internet Computer Protocol. Your agent lives in a canister — permanent, 
> cryptographically owned by you, waking itself on schedule, holding its own 
> encrypted credentials, and capable of earning ICP to fund its own operation.
> No cloud account. No subscription. No infrastructure to manage.

---

## What makes this different

Most "AI agents on blockchain" projects are smart contracts that call an LLM API.  
ZeroClaw ICP combines five ICP primitives that, together, produce something genuinely new:

| Primitive | What it enables |
|---|---|
| **ICP native timers** (`ic_cdk_timers`) | Agent wakes itself on schedule — no external cron, no keepalive |
| **Stable memory** (`ic-stable-structures`) | Agent memory is permanent and survives everything — no database needed |
| **HTTPS outcalls** | Agent reaches any external API — Anthropic, OpenAI, webhooks, anything |
| **vetKD threshold encryption** | Agent holds its own encrypted API keys — no operator can read them |
| **Internet Identity** | Agent is cryptographically owned by a person — no account to delete or ban |

---

## Architecture

Every agent deployment consists of three canisters:

```
┌─────────────────────────────────────────────────────────┐
│                    USER BROWSER                         │
│         Internet Identity auth → frontend canister      │
└─────────────────────┬───────────────────────────────────┘
                      │ Candid calls
┌─────────────────────▼───────────────────────────────────┐
│                  AGENT CANISTER                         │
│  Agent loop · Tool dispatch · Provider routing          │
│  ICP timer (cron/SOP) · Stable memory · Economics       │
│                                                         │
│   ic-llm (free)  ←→  HTTPS outcalls (Anthropic/OAI)    │
└──────────┬──────────────────────────────────────────────┘
           │ inter-canister call (principal-gated)
┌──────────▼──────────────────────────────────────────────┐
│                  VAULT CANISTER                         │
│  vetKD-inspired key store · Only agent can read keys    │
└─────────────────────────────────────────────────────────┘
```

### The two-tier inference model

```
Task arrives
    │
    ├── Simple / short context?
    │       └── ic-llm (Llama 3.1 8B / Qwen 3 32B)
    │           Cost: ~zero. Max 1000 tokens out.
    │           Good for: routing, classification, short tool calls
    │
    └── Complex / long context?
            └── HTTPS outcall → Anthropic / OpenAI
                Cost: ~0.49B cycles per call (~$0.0006)
                Good for: deep reasoning, long agentic loops, complex tool chains
```

---

## Quick start

### Prerequisites

- [dfx](https://internetcomputer.org/docs/current/developer-docs/getting-started/install/) (ICP SDK) — version 0.24.x recommended
- Rust stable toolchain with `wasm32-unknown-unknown` target
- Node.js 20+ (for frontend)

```bash
rustup target add wasm32-unknown-unknown
```

### Deploy locally

```bash
git clone https://github.com/YOUR_USERNAME/zeroclaw-icp.git
cd zeroclaw-icp
./scripts/deploy-local.sh
```

This will:
1. Start a local ICP replica (`dfx start`)
2. Deploy all three canisters
3. Print your local canister URLs
4. Open the frontend in your browser

### Deploy to mainnet

```bash
# Make sure you have ICP in your dfx identity wallet
dfx identity get-principal
./scripts/deploy-mainnet.sh
```

The deploy script prints your permanent canister IDs. Save them — your agent lives at those addresses forever.

---

## The self-funding agent model

> "Agents should do enough work to fund their own living, just like human beings."

ZeroClaw ICP is designed around the idea that an agent should be able to sustain itself economically. The framework ships with an `economics` module that implements:

```
timer fires
    │
    ├── Check task queue (free — ic-llm)
    │
    ├── Is there work worth doing?
    │       └── ic-llm decides (free)
    │
    ├── Do I have enough cycles?
    │       ├── YES → execute with Anthropic (~$0.002/turn)
    │       │           → did I generate value?
    │       │                   YES → convert earnings to cycles via CMC
    │       │                   NO  → log failure, adjust strategy
    │       └── NO  → execute degraded (ic-llm only)
    │               → alert owner via notification webhook
    │
    └── Sleep until next timer
```

An agent doing 100 turns/day burns approximately **150B cycles/day (~$0.20/day, ~$6/month)**.  
An agent that earns ICP by doing useful work can self-fund indefinitely.

---

## Canister interfaces

### Agent canister

```
// Send a message and get a response
chat(message: text) -> Result<text, text>

// Schedule a recurring task (cron expression + prompt)
add_sop(id: text, cron_expr: text, prompt: text) -> Result<(), text>

// Remove a scheduled task
remove_sop(id: text) -> Result<(), text>

// List all scheduled tasks
list_sops() -> vec SopEntry

// Check cycles balance
cycles_balance() -> nat64

// Get conversation history
get_history(limit: nat32) -> vec Message

// Update agent identity files
set_identity(identity_md: text, user_md: text, soul_md: text) -> Result<(), text>
```

### Vault canister

```
// Store an encrypted API key (controller or agent only)
store_key(name: text, value: blob) -> Result<(), text>

// Retrieve a key (agent principal only)
get_key(name: text) -> Result<blob, text>

// Set which agent canister can read keys (controller only)
set_agent_principal(agent: principal) -> Result<(), text>
```

---

## Cycles cost reference

| Operation | Cycles | USD (~) |
|---|---|---|
| Inter-canister call | ~10M | ~$0.000013 |
| HTTPS outcall (10KB response) | ~490M | ~$0.00064 |
| ic-llm call (Llama 3.1 8B) | ~10M | ~$0.000013 |
| Stable memory read (1KB) | ~1M | ~$0.0000013 |
| Canister creation | ~100B | ~$0.13 |
| 1 agent turn (simple, ic-llm) | ~20M | ~$0.000026 |
| 1 agent turn (complex, Anthropic) | ~1.5B | ~$0.002 |
| 100 turns/day (mixed) | ~150B/day | ~$0.20/day |

*Cycle costs current as of June 2026. See [docs/cycles-economics.md](docs/cycles-economics.md) for full budget planning guide.*

---

## Project structure

```
zeroclaw-icp/
├── canisters/
│   ├── agent/          # Core agent canister (Rust → Wasm)
│   │   └── src/
│   │       ├── lib.rs              # Canister entry, init/upgrade
│   │       ├── agent_loop.rs       # Observe → reason → plan → act
│   │       ├── provider.rs         # ic-llm + HTTPS outcall providers
│   │       ├── memory.rs           # ic-stable-structures memory layer
│   │       ├── sop.rs              # Standard Operating Procedures / cron
│   │       ├── economics.rs        # Cycles balance, earn(), degraded mode
│   │       ├── transform.rs        # HTTPS outcall header transform
│   │       └── tools/              # Tool implementations
│   ├── vault/          # Key store canister (Rust → Wasm)
│   └── frontend/       # React 18 + Vite + Internet Identity
├── scripts/
│   ├── deploy-local.sh
│   ├── deploy-mainnet.sh
│   └── create-agent.sh
├── docs/
│   ├── architecture.md
│   ├── cycles-economics.md
│   ├── setup-guide.md
│   └── self-funding-agents.md
├── tests/
├── dfx.json
└── Cargo.toml
```

---

## Relationship to ZeroClaw

This project draws architectural inspiration from [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) — a production Rust AI agent framework. We share the same design philosophy (trait-based, swappable backends, minimal overhead, agent-owned identity) but share zero code. ZeroClaw runs on Linux/macOS/Windows via a local binary. ZeroClaw ICP runs on the Internet Computer as Wasm canisters. They are complementary, not competing.

The trait boundaries in this framework (`Provider`, `Memory`, `Tool`) are designed to mirror ZeroClaw's architecture so that patterns, skills, and tool definitions can be ported between the two ecosystems with minimal friction.

---

## Roadmap

- [x] **v0.1.0** — Vault + Agent core + local deployment working
- [ ] **v0.2.0** — HTTPS outcall providers (Anthropic, OpenAI) + transform functions
- [ ] **v0.3.0** — Timer-based SOP/cron system + stable memory persistence
- [ ] **v0.4.0** — Frontend with Internet Identity auth + chat UI
- [ ] **v0.5.0** — Economics module: cycles balance checks, degraded mode, notification hooks
- [ ] **v0.6.0** — Factory canister: one-click multi-agent deployment
- [ ] **v1.0.0** — Mainnet stable, full docs, cycles estimator tool

---

## Contributing

ZeroClaw ICP is built in the open. The best contributions right now:

- **New tools** — implement the `Tool` trait in `canisters/agent/src/tools/`
- **New providers** — implement the `Provider` trait for new LLM backends
- **Cycles optimization** — tighter `max_response_bytes`, smarter transform functions
- **Documentation** — cost guides, setup walkthroughs, architecture explanations
- **Tests** — pocket-ic integration tests for the agent loop

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to get started.

---

## License

Dual-licensed for maximum openness:

| License | Use case |
|---|---|
| [MIT](LICENSE-MIT) | Open-source, research, personal use |
| [Apache 2.0](LICENSE-APACHE) | Patent protection, commercial deployment |

You may choose either license.

---

**ZeroClaw ICP** — Your agent. Your keys. Your cycles. Forever. 🦀
