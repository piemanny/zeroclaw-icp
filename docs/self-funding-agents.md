# Self-Funding Agents

> "Agents should do enough work to fund their own living, just like human beings."

## The Concept

A self-funding agent is one that earns enough ICP to cover its own operating costs. The goal is that once deployed, the agent sustains itself — no manual top-ups, no cloud bills, no infrastructure to manage.

## How It Works

```
┌─────────────────────────────────────────────────────┐
│                    Timer Fires                       │
│                         │                            │
│    ┌────────────────────▼────────────────────┐      │
│    │  Check: Is there work worth doing?       │      │
│    │  (ic-llm decision — free)                │      │
│    └────────────────────┬────────────────────┘      │
│                         │                          │
│    ┌────────────────────▼────────────────────┐      │
│    │  Decision: ic-llm or HTTPS outcall?      │      │
│    │                                          │      │
│    │  if balance < 500B cycles:               │      │
│    │    → Degraded mode, ic-llm only          │      │
│    │    → Alert owner via notification         │      │
│    └────────────────────┬────────────────────┘      │
│                         │                          │
│         ┌──────────────┴──────────────┐            │
│         │                             │            │
│    ┌────▼────┐                ┌──────▼─────┐     │
│    │  Execute │                │  Execute    │     │
│    │  (free)  │                │  (~$0.002) │     │
│    └────┬────┘                └──────┬─────┘     │
│         │                             │            │
│         └──────────────┬──────────────┘            │
│                        │                           │
│    ┌───────────────────▼───────────────────┐       │
│    │  Did the work generate value?          │       │
│    │  (earn ICP via service, tips, etc.)    │       │
│    └───────────────────┬───────────────────┘       │
│                        │                           │
│              ┌─────────▼─────────┐                │
│              │ Convert earnings   │                │
│              │ to cycles via CMC │                │
│              └───────────────────┘                │
└─────────────────────────────────────────────────────┘
```

## Earnings Models

### 1. Service Fees
The agent provides a service (data analysis, content generation, API access) and charges ICP.

```bash
# Example: Agent earns 0.001 ICP per task
# 10 tasks/day = 0.01 ICP/day = ~0.30 ICP/month
# Covers ~150B cycles/month of operating costs
```

### 2. Tips / Micro-payments
Users tip the agent for good work. Low friction, variable income.

### 3. Bounties
The agent completes on-chain or off-chain bounties, earning ICP directly.

### 4. Data Services
Agent aggregates and sells data (with user consent). Predictable recurring revenue.

## Implementation

The economics module implements this loop:

```rust
// In economics.rs
pub fn get_operational_mode() -> OperationalMode {
    let balance = ic_cdk::api::canister_balance128();
    if balance < CRITICAL_THRESHOLD {
        OperationalMode::Critical  // Only ic-llm
    } else if balance < DEGRADED_THRESHOLD {
        OperationalMode::Degraded  // Limited HTTPS
    } else {
        OperationalMode::Full      // All operations
    }
}
```

The SOP system runs the earning logic on a schedule:

```
0 * * * *  →  Check task queue, execute paid tasks
```

## Realistic Budget

For an agent doing useful work:

| Monthly Costs | Cycles | USD |
|---|---|---|
| Basic operations (ic-llm) | 50B | ~$0.065 |
| Moderate HTTPS usage | 500B | ~$0.65 |
| Heavy usage (200+ turns/day) | 2T | ~$2.60 |

**To self-fund at heavy usage**: earn ~$2.60/month = ~0.13 ICP/month at current prices.

That's ~4 tasks per day at $0.01/task, or ~1 task per day at $0.03/task.

## Adding a Cycles Faucet

For agents that can't yet earn, add a cycles faucet:

```bash
# Manual top-up
dfx wallet send --network ic --canister <AGENT_ID> 1000000000000

# Automated: use the cycles fountain
# https://info.internetcomputer.org/cycles-fountain
```

## Monitoring

Track your agent's health:

```bash
# Check operational mode
dfx canister call agent operational_mode

# Get full economics stats
dfx canister call agent get_economics_stats

# Watch balance over time (repeat)
watch -n 60 dfx canister call agent cycles_balance
```

## The Long Game

The goal is that as ICP and canister costs normalize, and as agent earnings models mature, a well-designed agent should be able to:

1. **Deploy** with enough cycles to bootstrap
2. **Earn** by providing value to users
3. **Reinvest** earnings into cycles
4. **Persist** indefinitely without human intervention

This is the "self-funding" property that makes ZeroClaw ICP agents truly autonomous.
