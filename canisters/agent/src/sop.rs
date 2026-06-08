use ic_cdk::api::time;
use ic_stable_structures::btreemap::BTreeMap;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, Storable, storable::Bound};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Debug, Clone, Serialize, Deserialize, candid::CandidType)]
pub struct SopEntry {
    pub id: String,
    pub cron_expr: String,
    pub prompt: String,
    pub last_run: u64,
    pub enabled: bool,
    pub description: String,
}

impl Storable for SopEntry {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(serde_json::to_vec(self).unwrap_or_default())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        serde_json::from_slice(&bytes).unwrap_or(SopEntry {
            id: String::new(),
            cron_expr: String::new(),
            prompt: String::new(),
            last_run: 0,
            enabled: false,
            description: String::new(),
        })
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 4000,
        is_fixed_size: false,
    };
}

struct State {
    sops: BTreeMap<String, SopEntry, Memory>,
}

impl State {
    fn new(memory: Memory) -> Self {
        State {
            sops: BTreeMap::init(memory),
        }
    }
}

thread_local! {
    static SOP_STATE: std::cell::RefCell<Option<State>> = std::cell::RefCell::new(None);
}

pub fn init_state() {
    let memory = ic_stable_structures::DefaultMemoryImpl::default();
    let mgr = MemoryManager::init(memory);
    let mem = mgr.get(MemoryId::new(1));
    SOP_STATE.with(|s| {
        *s.borrow_mut() = Some(State::new(mem));
    });
}

pub fn with_state<R>(f: impl FnOnce(&State) -> R) -> R {
    SOP_STATE.with(|s| f(s.borrow().as_ref().expect("SOP state not initialized")))
}

pub fn with_state_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
    SOP_STATE.with(|s| f(s.borrow_mut().as_mut().expect("SOP state not initialized")))
}

pub fn add_sop(id: String, cron_expr: String, prompt: String, description: String) -> Result<(), String> {
    with_state_mut(|state| {
        let entry = SopEntry {
            id: id.clone(),
            cron_expr: cron_expr.clone(),
            prompt: prompt.clone(),
            last_run: 0,
            enabled: true,
            description,
        };
        state.sops.insert(id, entry);
    });
    Ok(())
}

pub fn remove_sop(id: &str) -> Result<(), String> {
    with_state_mut(|state| {
        state.sops.remove(&id.to_string());
    });
    Ok(())
}

pub fn get_sop(id: &str) -> Option<SopEntry> {
    with_state(|state| state.sops.get(&id.to_string()))
}

pub fn list_sops() -> Vec<SopEntry> {
    with_state(|state| state.sops.values().collect())
}

pub fn update_last_run(id: &str) {
    with_state_mut(|state| {
        if let Some(sop) = state.sops.get(&id.to_string()) {
            let mut sop = sop;
            sop.last_run = time();
            state.sops.insert(id.to_string(), sop);
        }
    });
}

pub fn set_sop_enabled(id: &str, enabled: bool) -> Result<(), String> {
    let result: Result<(), String> = with_state_mut(|state| {
        if let Some(sop) = state.sops.get(&id.to_string()) {
            let mut sop = sop;
            sop.enabled = enabled;
            state.sops.insert(id.to_string(), sop);
            Ok(())
        } else {
            Err(format!("SOP '{}' not found", id))
        }
    });
    result
}

pub fn sop_count() -> usize {
    with_state(|state| state.sops.keys().count())
}

pub fn get_enabled_sops() -> Vec<SopEntry> {
    with_state(|state| {
        state.sops.values().filter(|s| s.enabled).collect()
    })
}

pub fn parse_cron_expr(cron_expr: &str) -> Result<u64, String> {
    let parts: Vec<&str> = cron_expr.split_whitespace().collect();
    if parts.len() < 5 {
        return Err("Invalid cron expression: need at least 5 fields".to_string());
    }

    let minutes: u64 = parts[0].parse().map_err(|_| "Invalid minutes field")?;
    let hours: u64 = parts[1].parse().map_err(|_| "Invalid hours field")?;
    let _day_of_month: &str = parts[2];
    let _month: &str = parts[3];
    let _day_of_week: &str = parts[4];

    let seconds = minutes * 60 + hours * 3600;
    if seconds == 0 {
        return Err("Interval must be at least 60 seconds".to_string());
    }

    Ok(seconds as u64)
}