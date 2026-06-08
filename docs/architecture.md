# Architecture

## Overview

ZeroClaw ICP is a 3-canister framework for deploying persistent, autonomous AI agents on the Internet Computer. Each agent deployment consists of three canisters that communicate via Candid.

## Canister Architecture

```
┌──────────────────────────────────────────────────────┐
│                   USER BROWSER                        │
│        Internet Identity → frontend canister           │
└─────────────────────┬────────────────────────────────┘
                      │ Candid (HTTPS + WebSocket)
┌─────────────────────▼────────────────────────────────┐
│                   AGENT CANISTER                     │
│                                                      │
│  ┌──────────────┐  ┌────────────┐  ┌────────────┐ │
│  │  Agent Loop  │  │  Provider  │  │   Tools    │ │
│  │  (observe →  │  │  routing   │  │  (recall,  │ │
│  │   reason →   │  │            │  │  store,    │ │
│  │   plan →     │  │ ic-llm ←→  │  │  fetch,    │ │
│  │   act)       │  │ HTTPS      │  │  notify)   │ │
│  └──────────────┘  └────────────┘  └────────────┘ │
│                                                      │
│  ┌──────────────┐  ┌────────────┐  ┌────────────┐ │
│  │   SOP/Cron   │  │ Economics  │  │  Stable    │ │
│  │   System     │  │  Module    │  │  Memory    │ │
│  │  (timers)    │  │  (cycles)  │  │  (BTreeMap)│ │
│  └──────────────┘  └────────────┘  └────────────┘ │
└─────────────────────┬──────────────────────────────┘
                      │ inter-canister call (principal-gated)
┌─────────────────────▼────────────────────────────────┐
│                   VAULT CANISTER                     │
│                                                      │
│  vetKD-inspired encrypted key store                  │
│  Only the paired agent principal can read keys       │
│  Controller sets the agent principal on deploy       │
└──────────────────────────────────────────────────────┘
```

## Agent Loop

The core agent loop implements the observe → reason → plan → act → verify pattern:

1. **Observe**: A message arrives via `chat()` or a timer fires
2. **Reason**: The provider decides whether to use ic-llm (free) or HTTPS outcall (paid)
3. **Plan**: For HTTPS calls, the agent builds the request with idempotency key
4. **Act**: Execute the provider call
5. **Verify**: Store the response in stable memory

## Provider Routing

Two-tier inference model:

| Task Type | Provider | Cost |
|---|---|---|
| Routing, classification, short tool calls | ic-llm (Llama 3.1 8B) | Free |
| Deep reasoning, long context, complex chains | Anthropic/OpenAI via HTTPS outcall | ~0.49B cycles/call |

Routing is based on:
- **Complexity**: Low (short input, simple task) vs High
- **Token estimate**: &lt;800 tokens → ic-llm, &ge;800 tokens → HTTPS outcall

## Stable Memory

All persistent data lives in ICP stable memory via `ic-stable-structures`:

| Data | Structure | Max Size |
|---|---|---|
| Conversations | BTreeMap&lt;ConversationId, Conversation&gt; | 100 conversations |
| Messages | Stored within Conversation | 1000 per conversation |
| SOPs | BTreeMap&lt;SopId, SopEntry&gt; | Unlimited |
| KV Store | BTreeMap&lt;Key, Value&gt; | 5KB per value |

No heap state survives upgrades — everything goes through Storable.

## Timer System

ICP native timers via `ic_cdk_timers`:

- **SOP queue ticker**: Every 60 seconds, checks enabled SOPs and fires due ones
- **Economics check**: Every 3600 seconds, checks balance and alerts owner if degraded
- Timers are re-registered in `post_upgrade` to survive canister upgrades

## HTTPS Outcalls

When the provider routes to Anthropic/OpenAI:

1. Build request with idempotency key (SHA256 of principal + session + message)
2. Call `http_request` with `max_response_bytes: 10_240`
3. Transform function strips non-deterministic headers before consensus
4. Parse response

One call costs ~490M cycles from subnet consensus.

## Secrets Management

API keys are stored in the vault canister:

1. Controller calls `vault.store_key("anthropic", encrypted_key)`
2. Agent fetches key via inter-canister call when needed
3. Only the agent's principal can call `vault.get_key()`

The vault does not implement actual vetKD — that requires chain-key cryptography unavailable in vanilla ic-cdk. For production, integrate with a vetKD-capable key management system.

## Upgrade Safety

All canister state survives upgrades:

- Stable memory (BTreeMaps) — automatic via ic-stable-structures
- Heap state (thread_local) — reinitialized in `post_upgrade`
- Timers — re-registered in `post_upgrade`

## Canister Interfaces

### Agent (agent.did)

Core: `chat`, `chat_in_conversation`, `get_history`, `cycles_balance`
SOPs: `add_sop`, `remove_sop`, `list_sops`, `set_sop_enabled`
Tools: `recall`, `store`, `forget`, `schedule_task`, `cancel_task`, `list_tasks`
Admin: `set_owner_principal`, `get_economics_stats`, `operational_mode`

### Vault (vault.did)

`store_key`, `get_key`, `set_agent_principal`, `get_agent_principal`
