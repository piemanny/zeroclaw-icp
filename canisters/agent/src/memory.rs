use ic_stable_structures::btree_map::BTreeMap;
use ic_stable_structures::memory_manager::MemoryManager;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{Storable, Memory};
use std::borrow::Cow;

type MemoryRegion = VirtualMemory<ic_cdk::api::memory::ICP_MEMORY>;

const MAX_MESSAGE_SIZE: u32 = 8000;
const MAX_CONVERSATIONS: u32 = 100;
const MAX_MESSAGES_PER_CONV: u32 = 1000;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
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
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
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
}

struct State {
    conversations: BTreeMap<String, Conversation, MemoryRegion>,
    message_buffers: BTreeMap<String, Vec<Message>, MemoryRegion>,
    kv_store: BTreeMap<String, String, MemoryRegion>,
}

impl State {
    fn new(memory: MemoryRegion) -> Self {
        State {
            conversations: BTreeMap::new(memory.clone()),
            message_buffers: BTreeMap::new(memory.clone()),
            kv_store: BTreeMap::new(memory),
        }
    }
}

thread_local! {
    static MEMORY_MANAGER: MemoryManager<MemoryRegion> =
        MemoryManager::init(ic_cdk::api::memory::ICP_MEMORY);

    static STATE: ic_stable_structures::cell_ref::Cell<State> = {
        let memory = MEMORY_MANAGER.with(|m| m.get(MemoryManager::MEMORY_ID));
        ic_stable_structures::cell_ref::Cell::new(State::new(memory))
    };
}

pub fn with_memory<R>(f: impl FnOnce(&MemoryRegion) -> R) -> R {
    MEMORY_MANAGER.with(|m| f(&m.get(MemoryManager::MEMORY_ID)))
}

pub fn with_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|s| f(s.borrow()))
}

pub fn with_state_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut *s.borrow_mut()))
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
            Some(mut conv) => {
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
    with_state(|state| state.conversations.get(&conversation_id.to_string()).cloned())
}

pub fn get_history(conversation_id: &str, limit: usize) -> Vec<Message> {
    with_state(|state| {
        state
            .conversations
            .get(&conversation_id.to_string())
            .map(|conv| {
                conv.messages
                    .iter()
                    .rev()
                    .take(limit)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    })
}

pub fn list_conversations() -> Vec<String> {
    with_state(|state| state.conversations.keys().cloned().collect())
}

pub fn conversation_count() -> u32 {
    with_state(|state| state.conversations.keys().len() as u32)
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
            .cloned()
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
    with_state(|state| state.kv_store.keys().cloned().collect())
}