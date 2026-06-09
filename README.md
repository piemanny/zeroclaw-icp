# 🦀 ZeroClaw ICP

**Persistent. Autonomous. Self-funding. Decentralized.**
Deploy AI agents on the Internet Computer that run forever — no server, no VPS, no maintenance.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache%202.0-blue.svg)](./LICENSE)
[![ICP](https://img.shields.io/badge/platform-Internet%20Computer-blue)](https://internetcomputer.org)
[![Rust](https://img.shields.io/badge/language-Rust-orange)](https://www.rust-lang.org)
[![Built with ic-cdk](https://img.shields.io/badge/built%20with-ic--cdk-purple)](https://crates.io/crates/ic-cdk)
[![Status: Alpha](https://img.shields.io/badge/status-alpha-red)](https://github.com/piemanny/zeroclaw-icp/releases)

---

## The Problem

You can run an AI agent on your own PC or server today. But then you're faced with a choice:

**Option A**: Pay for cloud hosting to guarantee uptime. You're back to renting someone else's infrastructure.

**Option B**: Maintain it yourself. Your agent dies when your machine goes offline.

Neither is acceptable for an agent that's supposed to work while you sleep.

---

## The Vision: ZeroClaw, But It Just Works Forever

ZeroClaw is a great AI agent runtime. It runs on your machine, works beautifully, and does what it says.

But what if it just... kept working? Forever? Without you? Without a server? Without you even thinking about it?

That's what we're building.

The same ZeroClaw experience — but underneath, it runs on the Internet Computer. It can't be shut down. It pays for its own hosting. It survives everything.

Users don't know ICP exists. They just know their agent has been running for 2 years without them touching it. And when they find out how... that's when minds get blown.

The goal is **zero friction, infinite permanence**.

---

## Why It Must Pay to Survive

An agent that funds itself. That earns enough to keep running. That dies if it can't.

Not because we're cruel — because that's what makes it real. If an agent can't sustain itself, it's just a toy. If it can, it's a business.

Fund it once. It earns. It pays its bills. It survives.

This is the difference between "a chatbot someone built" and "an entity that exists because it provides value."

---

## What Makes This Different

Most "AI agents on blockchain" projects are normal off-chain agents with a token
associated — the token and the chain have nothing to do with how the agent operates.
ZeroClaw ICP is different: the agent *is* the canister, running on-chain with no
off-chain dependencies, combining five ICP primitives that produce something
genuinely new:

| Primitive | What it Enables |
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
│         Internet Identity auth → frontend canister        │
└─────────────────────┬───────────────────────────────────┘
                      │ Candid calls
┌─────────────────────▼───────────────────────────────────┐
│                  AGENT CANISTER                         │
│  Agent loop · Tool dispatch · Provider routing           │
│  ICP timer (cron/SOP) · Stable memory · Economics      │
│                                                         │
│   ic-llm (free)  ←→  HTTPS outcalls (Anthropic/OAI)   │
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
                Cost: ~0.49B cycles/call (~$0.0006)
                Good for: deep reasoning, long agentic loops, complex tool chains
```

---

## Quick Start

### Prerequisites

- [dfx](https://internetcomputer.org/docs/current/developer-docs/getting-started/install/) (ICP SDK) — version 0.24.x recommended
- Rust stable toolchain with `wasm32-unknown-unknown` target
- Node.js 20+ (for frontend)

```bash
rustup target add wasm32-unknown-unknown
```

### Deploy locally

```bash
git clone https://github.com/piemanny/zeroclaw-icp.git
cd zeroclaw-icp
./scripts/deploy-local.sh
```

This will:
1. Build all canisters (Rust → Wasm32)
2. Start a local ICP replica
3. Deploy vault and agent canisters
4. Print your canister IDs

Then build the frontend:
```bash
cd canisters/frontend
cp .env.example .env
# Edit .env with your agent canister ID
npm install && npm run build
dfx deploy frontend --network local
```

---

## The self-funding agent model

> "Agents should do enough work to fund their own living, just like human beings."

ZeroClaw ICP is designed around the idea that an agent should be able to sustain itself economically:

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

## Canister interfaces

### Agent canister

```bash
# Send a message
dfx canister call agent chat '("Hello, ZeroClaw!")'

# Check cycles balance
dfx canister call agent cycles_balance

# Add a scheduled SOP
dfx canister call agent add_sop '("daily-briefing", "0 9 * * *", "Summary of my tasks", "Morning briefing")'

# List SOPs
dfx canister call agent list_sops

# Store/recall from persistent memory
dfx canister call agent store '("my-key", "my-value")'
dfx canister call agent recall '("my-key")'
```

### Vault canister

```bash
# Set agent principal (controller only)
dfx canister call vault set_agent_principal '(principal "...")'

# Store API key (controller or agent)
dfx canister call vault store_key '("anthropic", vec { 0x3a; 0xb2; ... })'

# Retrieve key (agent only)
dfx canister call vault get_key '("anthropic")'
```

---

## Project structure

```
zeroclaw-icp/
├── Cargo.toml                    # workspace
├── dfx.json                      # canister deployment config
├── canisters/
│   ├── agent/                    # Core agent canister (Rust → Wasm)
│   │   └── src/
│   │       ├── lib.rs              # Canister entry, init/upgrade
│   │       ├── agent_loop.rs       # Observe → reason → plan → act
│   │       ├── provider.rs         # ic-llm + HTTPS outcall providers
│   │       ├── memory.rs           # ic-stable-structures memory layer
│   │       ├── sop.rs              # SOP/cron system
│   │       ├── economics.rs        # Cycles balance, degraded mode
│   │       ├── outcall.rs          # HTTPS outcall logic
│   │       ├── transform.rs        # Header transform for outcalls
│   │       └── tools/              # Tool implementations
│   ├── vault/                      # Key store canister (Rust → Wasm)
│   └── frontend/                   # React 18 + Vite + Internet Identity
├── scripts/
│   ├── deploy-local.sh             # Local deployment
│   ├── deploy-mainnet.sh           # Mainnet deployment
│   └── create-agent.sh             # One-command agent spawn
└── docs/
    ├── architecture.md
    ├── cycles-economics.md
    ├── setup-guide.md
    └── self-funding-agents.md
```

---

## Relationship to ZeroClaw

This project draws architectural inspiration from [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) — a production Rust AI agent framework. We share the same design philosophy (trait-based, swappable backends, minimal overhead, agent-owned identity) but share zero code. ZeroClaw runs on Linux/macOS/Windows via a local binary. ZeroClaw ICP runs on the Internet Computer as Wasm canisters. They are complementary, not competing.

The trait boundaries in this framework (`Provider`, `Memory`, `Tool`) are designed to mirror ZeroClaw's architecture so that patterns, skills, and tool definitions can be ported between the two ecosystems with minimal friction.

---

## Roadmap

- [x] **v0.1.0** — Vault + Agent core + local deployment working
- [x] **v0.2.0** — HTTPS outcall providers (Anthropic, OpenAI) + transform functions
- [x] **v0.3.0** — Timer-based SOP/cron system + stable memory persistence
- [x] **v0.4.0** — Frontend with chat UI
- [ ] **v0.5.0** — Economics module: cycles balance checks, degraded mode, notification hooks
- [ ] **v0.6.0** — Decentralized AI integration (Bittensor, Akash, Render Network)
- [ ] **v0.7.0** — ICP chainfusion for cross-chain payments
- [ ] **v1.0.0** — Mainnet stable, full docs, autonomous business agents

---

## The Path Forward: Decentralized Infrastructure

The vision extends beyond ICP alone. We aim to build on decentralized infrastructure to achieve true self-sovereignty:

- **ICP** — The body. Hosts the agent runtime, canister state, and financial rails via chainfusion
- **Bittensor** — The brain. Decentralized, censorship-resistant AI models
- **Akash Network** — The blood flow. Decentralized GPU compute
- **Render Network** — GPU rendering and generative AI workloads
- **ICP chainfusion** — The financial system. Agents that send, receive, and manage their own funds

The specific providers and protocols in our research are illustrative examples of the kind of services we *could* integrate with — not a commitment to any particular platform. Our aim is to identify the most robust path to fully censorship-resistant, self-sovereign agents.

See [docs/decentralized-infrastructure-analysis.md](docs/decentralized-infrastructure-analysis.md) for our research on integration complexity.

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
| [MIT](./LICENSE-MIT) | Open-source, research, personal use |
| [Apache 2.0](./LICENSE-APACHE) | Patent protection, commercial deployment |

You may choose either license.

---

**ZeroClaw ICP** — Your agent. Your keys. Your cycles. Forever.