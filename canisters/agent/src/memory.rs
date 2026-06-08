use ic_stable_structures::btreemap::BTreeMap;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, Storable, storable::Bound};
use std::borrow::Cow;

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_MESSAGE_SIZE: u32 = 8000;
const MAX_CONVERSATIONS: u32 = 100;
const MAX_MESSAGES_PER_CONV: u32 = 1000;

#[derive(Clone, serde::Serialize, serde::Deserialize, candid::CandidType)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

impl Storable for Message {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(serde_json::to_vec(self).unwrap_or_default())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        serde_json::from_slice(&bytes).unwrap_or(Message {
            role: "user".to_string(),
            content: String::new(),
            timestamp: 0,
        })
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_MESSAGE_SIZE,
        is_fixed_size: false,
    };
}

#[derive(Clone, serde::Serialize, serde::Deserialize, candid::CandidType)]
pub struct Conversation {
    pub id: String,
    pub messages: Vec<Message>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Storable for Conversation {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(serde_json::to_vec(self).unwrap_or_default())
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        serde_json::from_slice(&bytes).unwrap_or(Conversation {
            id: String::new(),
            messages: vec![],
            created_at: 0,
            updated_at: 0,
        })
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_MESSAGE_SIZE * 10,
        is_fixed_size: false,
    };
}

struct State {
    conversations: BTreeMap<String, Conversation, Memory>,
    kv_store: BTreeMap<String, String, Memory>,
}

impl State {
    fn new(memory: Memory) -> Self {
        State {
            conversations: BTreeMap::init(memory.clone()),
            kv_store: BTreeMap::init(memory),
        }
    }
}

thread_local! {
    static MEMORY_MANAGER: std::cell::RefCell<MemoryManager<DefaultMemoryImpl>> =
        std::cell::RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static STATE: std::cell::RefCell<Option<State>> = std::cell::RefCell::new(None);
}

pub fn init_state() {
    MEMORY_MANAGER.with(|m| {
        let mem = m.borrow().get(MemoryId::new(0));
        STATE.with(|s| {
            *s.borrow_mut() = Some(State::new(mem));
        });
    });
}

pub fn with_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized")))
}

pub fn with_state_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized")))
}

pub fn add_message(conversation_id: &str, role: &str, content: &str) {
    with_state_mut(|state| {
        let timestamp = ic_cdk::api::time();
        let message = Message {
            role: role.to_string(),
            content: content.to_string(),
            timestamp,
        };
        match state.conversations.get(&conversation_id.to_string()) {
            Some(conv) => {
                let mut conv = conv;
                if conv.messages.len() >= MAX_MESSAGES_PER_CONV as usize {
                    conv.messages.remove(0);
                }
                conv.messages.push(message);
                conv.updated_at = timestamp;
                state.conversations.insert(conversation_id.to_string(), conv);
            }
            None => {
                let conv = Conversation {
                    id: conversation_id.to_string(),
                    messages: vec![message],
                    created_at: timestamp,
                    updated_at: timestamp,
                };
                state.conversations.insert(conversation_id.to_string(), conv);
            }
        }
    });
}

pub fn get_conversation(conversation_id: &str) -> Option<Conversation> {
    with_state(|state| state.conversations.get(&conversation_id.to_string()))
}

pub fn get_history(conversation_id: &str, limit: usize) -> Vec<Message> {
    with_state(|state| {
        state
            .conversations
            .get(&conversation_id.to_string())
            .map(|conv| {
                let msgs: Vec<_> = conv.messages.iter().rev().take(limit).cloned().collect();
                msgs.into_iter().rev().collect()
            })
            .unwrap_or_default()
    })
}

pub fn list_conversations() -> Vec<String> {
    with_state(|state| state.conversations.keys().collect())
}

pub fn conversation_count() -> u32 {
    with_state(|state| state.conversations.keys().count() as u32)
}

pub fn format_history_for_llm(conversation_id: &str) -> String {
    get_history(conversation_id, 50)
        .into_iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn get_memory_value(key: &str) -> String {
    with_state(|state| {
        state
            .kv_store
            .get(&key.to_string())
            .unwrap_or_default()
    })
}

pub fn set_memory_value(key: &str, value: &str) {
    with_state_mut(|state| {
        state.kv_store.insert(key.to_string(), value.to_string());
    });
}

pub fn delete_memory_value(key: &str) {
    with_state_mut(|state| {
        state.kv_store.remove(&key.to_string());
    });
}

pub fn list_memory_keys() -> Vec<String> {
    with_state(|state| state.kv_store.keys().collect())
}