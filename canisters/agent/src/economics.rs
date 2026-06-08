use ic_cdk::api::call::notify::notify;
use ic_cdk::export::Principal;

const MINIMUM_CYCLES_BALANCE: u128 = 500_000_000_000;
const CRITICAL_CYCLES_BALANCE: u128 = 100_000_000_000;
const DEGRADED_MODE_THRESHOLD: u128 = 200_000_000_000;

#[derive(Debug, Clone)]
pub enum OperationalMode {
    Full,
    Degraded,
    Critical,
}

pub fn get_operational_mode() -> OperationalMode {
    let balance = ic_cdk::api::canister_balance128();

    if balance < CRITICAL_CYCLES_BALANCE as u128 {
        OperationalMode::Critical
    } else if balance < DEGRADED_MODE_THRESHOLD as u128 {
        OperationalMode::Degraded
    } else {
        OperationalMode::Full
    }
}

pub fn check_balance_for_outcall() -> bool {
    let balance = ic_cdk::api::canister_balance128();
    balance >= MINIMUM_CYCLES_BALANCE
}

pub fn get_balance_u128() -> u128 {
    ic_cdk::api::canister_balance128()
}

pub fn get_balance_nat64() -> u64 {
    let balance = ic_cdk::api::canister_balance128();
    balance as u64
}

pub fn estimate_outcall_cost(provider: &str, tokens: usize) -> u128 {
    let base: u128 = match provider {
        "anthropic" => 490_000_000,
        "openai" => 450_000_000,
        "ic-llm" => 10_000_000,
        _ => 500_000_000,
    };
    let per_token = (tokens as u128).saturating_mul(100_000);
    base.saturating_add(per_token)
}

pub fn should_use_degraded_mode() -> bool {
    let mode = get_operational_mode();
    matches!(mode, OperationalMode::Degraded | OperationalMode::Critical)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EconomicsStats {
    pub balance: u64,
    pub operational_mode: String,
    pub can_execute_outcall: bool,
    pub estimated_cycles_per_day: u64,
}

pub fn get_stats() -> EconomicsStats {
    let balance = get_balance_u128();
    let mode = get_operational_mode();
    let can_outcall = check_balance_for_outcall();

    EconomicsStats {
        balance: balance as u64,
        operational_mode: format!("{:?}", mode),
        can_execute_outcall: can_outcall,
        estimated_cycles_per_day: 150_000_000_000,
    }
}

pub async fn notify_owner(message: &str, recipient: Principal) -> Result<(), String> {
    let payload = format!("ZeroClaw Economics Alert: {}", message);
    let _ = notify(recipient, "handle_notification", &(payload.as_bytes().to_vec())).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn format_balance(balance: u128) -> String {
    if balance >= 1_000_000_000_000 {
        format!("{:.2}T", balance as f64 / 1_000_000_000_000.0)
    } else if balance >= 1_000_000_000 {
        format!("{:.2}B", balance as f64 / 1_000_000_000.0)
    } else if balance >= 1_000_000 {
        format!("{:.2}M", balance as f64 / 1_000_000.0)
    } else {
        format!("{} cycles", balance)
    }
}
