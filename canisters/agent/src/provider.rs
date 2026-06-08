use crate::outcall::{anthropic_completion, openai_completion, AnthropicMessage, OpenAiMessage};
use ic_llm::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Backend {
    Anthropic,
    OpenAI,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Complexity {
    Low,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub prompt: String,
    pub complexity: Complexity,
    pub token_estimate: usize,
    pub conversation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub provider: String,
    pub model: String,
    pub tokens_used: usize,
}

pub struct IcLlmProvider {
    model: Model,
}

impl IcLlmProvider {
    pub fn new(model: Model) -> Self {
        Self { model }
    }

    pub async fn complete(&self, prompt: &str, _max_tokens: u32) -> Result<LlmResponse, String> {
        let model_name = match self.model {
            Model::Llama3_1_8B => "Llama3.1-8B".to_string(),
            Model::Qwen3_32B => "Qwen3-32B".to_string(),
            Model::Llama4Scout => "Llama4-Scout".to_string(),
        };

        let model_for_prompt = match self.model {
            Model::Llama3_1_8B => Model::Llama3_1_8B,
            Model::Qwen3_32B => Model::Qwen3_32B,
            Model::Llama4Scout => Model::Llama4Scout,
        };

        let content = ic_llm::prompt(model_for_prompt, prompt).await;

        Ok(LlmResponse {
            content,
            provider: "ic-llm".to_string(),
            model: model_name,
            tokens_used: prompt.len() / 4,
        })
    }

    pub fn name(&self) -> String {
        match self.model {
            Model::Llama3_1_8B => "ic-llm/Llama3.1-8B".to_string(),
            Model::Qwen3_32B => "ic-llm/Qwen3-32B".to_string(),
            Model::Llama4Scout => "ic-llm/Llama4-Scout".to_string(),
        }
    }
}

pub struct HttpsOutcallProvider {
    backend: Backend,
    api_key: Option<String>,
}

impl HttpsOutcallProvider {
    pub fn new(backend: Backend, api_key: Option<String>) -> Self {
        Self { backend, api_key }
    }

    pub async fn complete(
        &self,
        messages: &[AnthropicMessage],
        system: Option<&str>,
        max_tokens: u32,
        principal: &str,
        session: &str,
    ) -> Result<LlmResponse, String> {
        let api_key = self.api_key.as_ref().ok_or("No API key configured")?;

        match self.backend {
            Backend::Anthropic => {
                let (content, tokens) = anthropic_completion(
                    api_key,
                    "claude-sonnet-4-20250514",
                    messages,
                    system,
                    max_tokens,
                    principal,
                    session,
                )
                .await?;

                Ok(LlmResponse {
                    content,
                    provider: "Anthropic".to_string(),
                    model: "claude-sonnet-4-20250514".to_string(),
                    tokens_used: tokens,
                })
            }
            Backend::OpenAI => {
                let openai_messages: Vec<OpenAiMessage> = messages
                    .iter()
                    .map(|m| OpenAiMessage {
                        role: m.role.clone(),
                        content: m.content.clone(),
                    })
                    .collect();

                let (content, tokens) = openai_completion(
                    api_key,
                    "gpt-4o-mini",
                    &openai_messages,
                    max_tokens,
                    principal,
                    session,
                )
                .await?;

                Ok(LlmResponse {
                    content,
                    provider: "OpenAI".to_string(),
                    model: "gpt-4o-mini".to_string(),
                    tokens_used: tokens,
                })
            }
        }
    }

    pub fn name(&self) -> &str {
        match self.backend {
            Backend::Anthropic => "https/Anthropic",
            Backend::OpenAI => "https/OpenAI",
        }
    }
}

pub fn select_provider_for_task(task: &Task) -> (String, Model) {
    if task.complexity == Complexity::Low && task.token_estimate < 800 {
        ("ic-llm".to_string(), Model::Llama3_1_8B)
    } else {
        ("https-outcall".to_string(), Model::Llama3_1_8B)
    }
}

pub async fn execute_with_ic_llm(
    model: Model,
    prompt: &str,
    _max_tokens: u32,
) -> Result<LlmResponse, String> {
    let provider = IcLlmProvider::new(model);
    provider.complete(prompt, 1000).await
}

pub async fn execute_with_https_outcall(
    backend: Backend,
    api_key: Option<String>,
    messages: &[AnthropicMessage],
    system: Option<&str>,
    max_tokens: u32,
    principal: &str,
    session: &str,
) -> Result<LlmResponse, String> {
    let provider = HttpsOutcallProvider::new(backend, api_key);
    provider
        .complete(messages, system, max_tokens, principal, session)
        .await
}