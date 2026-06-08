use crate::memory;
use crate::outcall::AnthropicMessage;
use crate::provider::{execute_with_provider, select_provider_for_task, Complexity, Provider, Task};
use std::sync::atomic::{AtomicU64, Ordering};

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_session_id() -> String {
    let count = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let principal = ic_cdk::api::caller().to_string();
    format!("{}-{}", principal, count)
}

pub fn build_system_prompt() -> String {
    r#"You are ZeroClaw, an autonomous AI agent running on the Internet Computer Protocol (ICP).

Your traits:
- You are persistent — your memory survives canister upgrades
- You operate autonomously using ICP native timers (no external cron)
- You hold your own encrypted API keys via the vault canister
- You can call external tools and make HTTP requests via ICP HTTPS outcalls
- You are cryptographically owned by your user via Internet Identity

Guidelines:
- Be concise and action-oriented
- When uncertain, use ic-llm for routing decisions (free)
- Log your reasoning before taking action
- Always check cycles balance before expensive operations

Identity files you maintain:
- IDENTITY.md: Your core personality and values
- USER.md: Your user's preferences and context
- SOUL.md: Your principles and operating constraints
"#
    .to_string()
}

fn build_messages(conversation_id: &str, user_message: &str) -> (Vec<AnthropicMessage>, Option<String>) {
    let history = memory::get_history(conversation_id, 50);

    let mut messages: Vec<AnthropicMessage> = history
        .into_iter()
        .map(|m| AnthropicMessage {
            role: m.role,
            content: m.content,
        })
        .collect();

    messages.push(AnthropicMessage {
        role: "user".to_string(),
        content: user_message.to_string(),
    });

    let system = Some(build_system_prompt());
    (messages, system)
}

pub async fn run_single_turn(
    conversation_id: &str,
    user_message: &str,
) -> Result<String, String> {
    let session_id = next_session_id();
    let principal = ic_cdk::api::caller().to_string();

    memory::add_message(conversation_id, "user", user_message);

    let (messages, system) = build_messages(conversation_id, user_message);

    let token_estimate = messages.iter().map(|m| m.content.len()).sum::<usize>() / 4;
    let complexity = if token_estimate < 800 {
        Complexity::Low
    } else {
        Complexity::High
    };

    let task = Task {
        id: session_id.clone(),
        prompt: String::new(),
        complexity,
        token_estimate,
        conversation_id: conversation_id.to_string(),
    };

    let provider = select_provider_for_task(&task);

    let response = execute_with_provider(
        &provider,
        &messages,
        system.as_deref(),
        1000,
        &principal,
        &session_id,
    )
    .await?;

    memory::add_message(conversation_id, "assistant", &response.content);

    Ok(response.content)
}

pub async fn run_agent_turn(
    conversation_id: &str,
    user_message: &str,
) -> Result<AgentResponse, String> {
    let session_id = next_session_id();
    let principal = ic_cdk::api::caller().to_string();

    memory::add_message(conversation_id, "user", user_message);

    let (messages, system) = build_messages(conversation_id, user_message);

    let token_estimate = messages.iter().map(|m| m.content.len()).sum::<usize>() / 4;
    let complexity = if token_estimate < 800 {
        Complexity::Low
    } else {
        Complexity::High
    };

    let task = Task {
        id: session_id.clone(),
        prompt: String::new(),
        complexity,
        token_estimate,
        conversation_id: conversation_id.to_string(),
    };

    let provider = select_provider_for_task(&task);

    let response = execute_with_provider(
        &provider,
        &messages,
        system.as_deref(),
        1000,
        &principal,
        &session_id,
    )
    .await?;

    memory::add_message(conversation_id, "assistant", &response.content);

    Ok(AgentResponse {
        content: response.content,
        provider_name: format!("{:?}", provider),
        tokens_used: response.tokens_used,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentResponse {
    pub content: String,
    pub provider_name: String,
    pub tokens_used: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
    pub tool: String,
    pub result: String,
    pub success: bool,
}
