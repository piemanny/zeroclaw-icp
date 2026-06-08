use candid::Principal;
use ic_cdk::caller;
use ic_stable_structures::btreemap::BTreeMap;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, Storable, storable::Bound};
use std::borrow::Cow;

const MAX_KEY_SIZE: u32 = 100;
const MAX_VALUE_SIZE: u32 = 5000;

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct KeyEntry {
    value: Vec<u8>,
}

impl Storable for KeyEntry {
    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(serde_json::to_vec(self).unwrap_or_default())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        serde_json::from_slice(&bytes).unwrap_or(KeyEntry { value: vec![] })
    }
}

struct State {
    keys: BTreeMap<String, KeyEntry, Memory>,
    agent_principal: Option<Principal>,
    controller: Principal,
}

impl State {
    fn new(memory: Memory) -> Self {
        State {
            keys: BTreeMap::init(memory),
            agent_principal: None,
            controller: caller(),
        }
    }
}

thread_local! {
    static MEMORY_MANAGER: std::cell::RefCell<MemoryManager<DefaultMemoryImpl>> =
        std::cell::RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static STATE: std::cell::RefCell<Option<State>> = std::cell::RefCell::new(None);
}

fn with_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized")))
}

fn with_state_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized")))
}

fn is_controller() -> bool {
    caller() == with_state(|s| s.controller)
}

fn is_agent() -> bool {
    with_state(|s| s.agent_principal.map(|p| p == caller()).unwrap_or(false))
}

fn is_authorized() -> bool {
    is_controller() || is_agent()
}

#[ic_cdk::init]
fn init() {
    MEMORY_MANAGER.with(|m| {
        let mem = m.borrow().get(MemoryId::new(0));
        STATE.with(|s| {
            *s.borrow_mut() = Some(State::new(mem));
        });
    });
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    if STATE.with(|s| s.borrow().is_none()) {
        MEMORY_MANAGER.with(|m| {
            let mem = m.borrow().get(MemoryId::new(0));
            STATE.with(|s| {
                *s.borrow_mut() = Some(State::new(mem));
            });
        });
    }
}

#[ic_cdk::update]
fn set_agent_principal(agent: Principal) -> Result<(), String> {
    if !is_controller() {
        return Err("Only controller can set agent principal".to_string());
    }
    with_state_mut(|s| {
        s.agent_principal = Some(agent);
    });
    Ok(())
}

#[ic_cdk::query]
fn get_agent_principal() -> Option<Principal> {
    with_state(|s| s.agent_principal)
}

#[ic_cdk::update]
fn store_key(name: String, value: Vec<u8>) -> Result<(), String> {
    if !is_authorized() {
        return Err("Only controller or agent principal can store keys".to_string());
    }
    if name.len() > MAX_KEY_SIZE as usize {
        return Err(format!("Key name exceeds {} bytes", MAX_KEY_SIZE));
    }
    if value.len() > MAX_VALUE_SIZE as usize {
        return Err(format!("Value exceeds {} bytes", MAX_VALUE_SIZE));
    }
    let entry = KeyEntry { value };
    with_state_mut(|s| {
        s.keys.insert(name, entry);
    });
    Ok(())
}

#[ic_cdk::query]
fn get_key(name: String) -> Result<Vec<u8>, String> {
    if !is_agent() {
        return Err("Only agent principal can retrieve keys".to_string());
    }
    with_state(|s| s.keys.get(&name).map(|entry| entry.value.clone()))
        .ok_or_else(|| format!("Key '{}' not found", name))
}

#[ic_cdk::update]
fn remove_key(name: String) -> Result<(), String> {
    if !is_authorized() {
        return Err("Only controller or agent principal can remove keys".to_string());
    }
    with_state_mut(|s| {
        s.keys.remove(&name);
    });
    Ok(())
}