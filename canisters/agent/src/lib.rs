mod agent_loop;
mod economics;
mod memory;
mod outcall;
mod provider;
mod sop;
mod tools;
mod transform;

use economics::{self, EconomicsStats, OperationalMode};
use ic_cdk::api::time;
use ic_cdk::timers::set_timer_interval;
use ic_cdk::export::Principal;
use memory::{self, Conversation, Message};
use sop::SopEntry;
use std::cell::RefCell;
use std::time::Duration;
use tools::{self, ToolResult};

const TIMER_INTERVAL_SECS: u64 = 60;

thread_local! {
    static CONTROLLER: RefCell<Option<Principal>> = RefCell::new(None);
    static DEFAULT_CONVERSATION_ID: RefCell<String> = RefCell::new("default".to_string());
    static OWNER_PRINCIPAL: RefCell<Option<Principal>> = RefCell::new(None);
    static TIMER_RUNNING: RefCell<bool> = RefCell::new(false);
}

fn get_controller() -> Option<Principal> {
    CONTROLLER.with(|c| c.borrow().clone())
}

fn get_caller() -> Principal {
    ic_cdk::api::caller()
}

fn is_controller() -> bool {
    get_controller()
        .map(|c| c == get_caller())
        .unwrap_or(false)
}

fn tick_sop_queue() {
    let enabled_sops = sop::get_enabled_sops();
    let now = time();

    for sop_entry in enabled_sops {
        let interval = sop::parse_cron_expr(&sop_entry.cron_expr).unwrap_or(60);
        let should_run = sop_entry.last_run == 0 || (now - sop_entry.last_run) >= interval;

        if should_run {
            ic_cdk::spawn(async {
                let conv_id = format!("sop-{}", sop_entry.id);
                let _ = agent_loop::run_agent_turn(&conv_id, &sop_entry.prompt).await;
            });
            sop::update_last_run(&sop_entry.id);
        }
    }
}

fn tick_economics() {
    let mode = economics::get_operational_mode();

    match mode {
        OperationalMode::Critical => {
            if let Some(owner) = OWNER_PRINCIPAL.with(|p| p.borrow().clone()) {
                let _ = ic_cdk::notify(
                    owner,
                    "handle_notification",
                    &("CRITICAL: Cycles balance critically low".as_bytes().to_vec()),
                );
            }
        }
        OperationalMode::Degraded => {
            if let Some(owner) = OWNER_PRINCIPAL.with(|p| p.borrow().clone()) {
                let balance = economics::format_balance(economics::get_balance_u128());
                let msg = format!("WARNING: Running in degraded mode. Balance: {}", balance);
                let _ = ic_cdk::notify(
                    owner,
                    "handle_notification",
                    &(msg.as_bytes().to_vec()),
                );
            }
        }
        OperationalMode::Full => {}
    }
}

fn start_timers() {
    let already_running = TIMER_RUNNING.with(|t| *t.borrow());
    if already_running {
        return;
    }

    set_timer_interval(Duration::from_secs(TIMER_INTERVAL_SECS), || {
        tick_sop_queue();
    });

    set_timer_interval(Duration::from_secs(3600), || {
        tick_economics();
    });

    TIMER_RUNNING.with(|t| *t.borrow_mut() = true);
}

#[ic_cdk::init]
fn init() {
    CONTROLLER.with(|c| {
        *c.borrow_mut() = Some(get_caller());
    });
    DEFAULT_CONVERSATION_ID.with(|id| {
        *id.borrow_mut() = format!("conv-{}", get_caller().to_text());
    });
    OWNER_PRINCIPAL.with(|p| {
        *p.borrow_mut() = Some(get_caller());
    });
    start_timers();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    if get_controller().is_none() {
        CONTROLLER.with(|c| {
            *c.borrow_mut() = Some(get_caller());
        });
    }
    if OWNER_PRINCIPAL.with(|p| p.borrow().is_none()) {
        OWNER_PRINCIPAL.with(|p| {
            *p.borrow_mut() = Some(get_caller());
        });
    }
    TIMER_RUNNING.with(|t| *t.borrow_mut() = false);
    start_timers();
}

#[ic_cdk::update]
async fn chat(message: String) -> Result<String, String> {
    let conv_id = DEFAULT_CONVERSATION_ID.with(|id| id.borrow().clone());
    run_agent_turn(&conv_id, &message).await.map(|r| r.content)
}

#[ic_cdk::update]
async fn chat_in_conversation(conversation_id: String, message: String) -> Result<String, String> {
    run_agent_turn(&conversation_id, &message).await.map(|r| r.content)
}

use agent_loop::run_agent_turn;

#[ic_cdk::query]
fn get_history(limit: u32) -> Vec<Message> {
    let conv_id = DEFAULT_CONVERSATION_ID.with(|id| id.borrow().clone());
    memory::get_history(&conv_id, limit as usize)
}

#[ic_cdk::query]
fn get_conversation(conversation_id: String) -> Option<Conversation> {
    memory::get_conversation(&conversation_id)
}

#[ic_cdk::query]
fn list_conversations() -> Vec<String> {
    memory::list_conversations()
}

#[ic_cdk::query]
fn conversation_count() -> u32 {
    memory::conversation_count()
}

#[ic_cdk::query]
fn cycles_balance() -> u64 {
    economics::get_balance_nat64()
}

#[ic_cdk::query]
fn get_economics_stats() -> EconomicsStats {
    economics::get_stats()
}

#[ic_cdk::query]
fn operational_mode() -> String {
    format!("{:?}", economics::get_operational_mode())
}

#[ic_cdk::update]
fn set_default_conversation(conversation_id: String) -> Result<(), String> {
    if !is_controller() {
        return Err("Only controller can set default conversation".to_string());
    }
    DEFAULT_CONVERSATION_ID.with(|id| {
        *id.borrow_mut() = conversation_id;
    });
    Ok(())
}

#[ic_cdk::update]
fn clear_history(conversation_id: String) -> Result<(), String> {
    if !is_controller() {
        return Err("Only controller can clear history".to_string());
    }
    let _ = memory::list_conversations();
    Ok(())
}

#[ic_cdk::update]
fn add_sop(id: String, cron_expr: String, prompt: String, description: String) -> Result<(), String> {
    sop::add_sop(id, cron_expr, prompt, description)
}

#[ic_cdk::update]
fn remove_sop(id: String) -> Result<(), String> {
    sop::remove_sop(&id)
}

#[ic_cdk::query]
fn list_sops() -> Vec<SopEntry> {
    sop::list_sops()
}

#[ic_cdk::query]
fn sop_count() -> usize {
    sop::sop_count()
}

#[ic_cdk::update]
fn set_sop_enabled(id: String, enabled: bool) -> Result<(), String> {
    sop::set_sop_enabled(&id, enabled)
}

#[ic_cdk::update]
fn recall(key: String) -> ToolResult {
    tools::recall(&key)
}

#[ic_cdk::update]
fn store(key: String, value: String) -> ToolResult {
    tools::store(&key, &value)
}

#[ic_cdk::update]
fn forget(key: String) -> ToolResult {
    tools::forget(&key)
}

#[ic_cdk::query]
fn list_memory_keys() -> Vec<String> {
    memory::list_memory_keys()
}

#[ic_cdk::update]
fn schedule_task(id: String, cron_expr: String, payload: String) -> ToolResult {
    tools::schedule_task(id, cron_expr, payload)
}

#[ic_cdk::update]
fn cancel_task(id: String) -> ToolResult {
    tools::cancel_task(&id)
}

#[ic_cdk::query]
fn list_tasks() -> Vec<tools::timer::ScheduledTask> {
    tools::list_tasks()
}

#[ic_cdk::update]
fn send_notification(message: String) -> ToolResult {
    tools::send_notification(&message)
}

#[ic_cdk::update]
fn set_owner_principal(principal: Principal) -> Result<(), String> {
    if !is_controller() {
        return Err("Only controller can set owner principal".to_string());
    }
    OWNER_PRINCIPAL.with(|p| {
        *p.borrow_mut() = Some(principal);
    });
    Ok(())
}

#[ic_cdk::query]
fn whoami() -> Principal {
    get_caller()
}

#[ic_cdk::query]
fn get_transform_func_name() -> String {
    "transform".to_string()
}

#[ic_cdk::query]
fn handle_notification(payload: Vec<u8>) -> Result<(), String> {
    let _msg = String::from_utf8(payload).map_err(|e| e.to_string())?;
    Ok(())
}
