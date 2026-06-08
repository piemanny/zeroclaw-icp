# Cycles Economics

## Cost Reference

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

*Cycle prices as of June 2026. ICP price ~$1.30/TC.*

## Operational Modes

The economics module tracks balance and operates in three modes:

| Mode | Balance Range | Behavior |
|---|---|---|
| **Full** | &gt;200B cycles | All operations enabled, HTTPS outcalls allowed |
| **Degraded** | 100-200B cycles | Alerts owner, limits HTTPS outcalls |
| **Critical** | &lt;100B cycles | Only ic-llm calls, owner notified every hour |

## Daily Burn Rate

Typical agent daily burn by usage pattern:

| Usage Pattern | Daily Cycles | Monthly USD |
|---|---|---|
| 10 turns/day, ic-llm only | ~200M | ~$0.008 |
| 50 turns/day, mostly ic-llm | ~1B | ~$0.04 |
| 100 turns/day, mixed | ~150B | ~$0.20 |
| 200 turns/day, mostly HTTPS | ~300B | ~$0.40 |

## Self-Funding Budget

For a self-funding agent that earns ICP:

| Monthly Earnings | Self-Funding Capacity | Notes |
|---|---|---|
| 0.1 ICP/month | ~200B cycles | Covers basic ic-llm usage |
| 0.5 ICP/month | ~1T cycles | Covers moderate HTTPS usage |
| 1 ICP/month | ~2T cycles | Full operations, 100+ turns/day |
| 5 ICP/month | ~10T cycles | Heavy usage, multiple SOPs |

## Adding Cycles

```bash
# Check balance
dfx canister status <AGENT_ID> --network ic

# Add cycles from your wallet
dfx wallet send --network ic --canister <AGENT_ID> 1000000000000

# Or use the cycles fountain (CMC)
# https://info.internetcomputer.org/cycles-fountain
```

## Cost Optimization Tips

1. **Use ic-llm for routing decisions** — it's free and fast for classification
2. **Set appropriate max_response_bytes** — smaller responses cost less
3. **Batch tool calls** — multiple actions in one turn instead of multiple turns
4. **Use degraded mode intentionally** — when balance is low, route to ic-llm only
5. **Monitor with `get_economics_stats`** — track balance and adjust behavior

## Balance Alert Thresholds

Set these in your monitoring:

```rust
const WARNING_THRESHOLD: u128 = 200_000_000_000;  // 200B — degraded mode
const CRITICAL_THRESHOLD: u128 = 100_000_000_000;  // 100B — ic-llm only
const MIN_OUTCALL_BALANCE: u128 = 500_000_000_000;  // 500B — allow HTTPS outcall
```
