use ic_cdk::api::caller;
use ic_cdk::export::Principal;
use ic_stable_structures::btree_map::BTreeMap;
use ic_stable_structures::Storable;
use std::borrow::Cow;

const MAX_KEY_SIZE: u32 = 100;
const MAX_VALUE_SIZE: u32 = 5000;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct KeyEntry {
    value: Vec<u8>,
}

impl Storable for KeyEntry {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(serde_json::to_vec(self).unwrap_or_default())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        serde_json::from_slice(&bytes).unwrap_or(KeyEntry { value: vec![] })
    }
}

struct State {
    keys: BTreeMap<String, KeyEntry>,
    agent_principal: Option<Principal>,
    controller: Principal,
}

impl State {
    fn new() -> Self {
        State {
            keys: BTreeMap::new(),
            agent_principal: None,
            controller: caller(),
        }
    }
}

thread_local! {
    static STATE: ic_stable_structures::cell_ref::Cell<State> =
        ic_stable_structures::cell_ref::Cell::new(State::new());
}

fn is_controller() -> bool {
    caller() == STATE.with(|s| s.borrow().controller)
}

fn is_agent() -> bool {
    let state = STATE.with(|s| s.borrow());
    state.agent_principal.map(|p| p == caller()).unwrap_or(false)
}

fn is_authorized() -> bool {
    is_controller() || is_agent()
}

#[ic_cdk::init]
fn init() {
    STATE.with(|s| {
        *s.borrow_mut() = State::new();
    });
}

#[ic_cdk::update]
fn set_agent_principal(agent: Principal) -> Result<(), String> {
    if !is_controller() {
        return Err("Only controller can set agent principal".to_string());
    }
    STATE.with(|s| {
        s.borrow_mut().agent_principal = Some(agent);
    });
    Ok(())
}

#[ic_cdk::query]
fn get_agent_principal() -> Option<Principal> {
    STATE.with(|s| s.borrow().agent_principal)
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
    STATE.with(|s| {
        s.borrow_mut().keys.insert(name, entry);
    });
    Ok(())
}

#[ic_cdk::query]
fn get_key(name: String) -> Result<Vec<u8>, String> {
    if !is_agent() {
        return Err("Only agent principal can retrieve keys".to_string());
    }
    STATE
        .with(|s| s.borrow().keys.get(&name).cloned())
        .map(|entry| entry.value)
        .ok_or_else(|| format!("Key '{}' not found", name))
}

#[ic_cdk::update]
fn remove_key(name: String) -> Result<(), String> {
    if !is_authorized() {
        return Err("Only controller or agent principal can remove keys".to_string());
    }
    STATE.with(|s| {
        s.borrow_mut().keys.remove(&name);
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_storage() {
        let state = State::new();
        assert!(state.keys.get(&"test".to_string()).is_none());
    }
}