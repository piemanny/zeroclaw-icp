use crate::outcall::{self, anthropic_completion, openai_completion};
use crate::transform::transform;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Model {
    Llama3_1_8B,
    Qwen3_32B,
    Llama4Scout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Backend {
    Anthropic,
    OpenAI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provider {
    IcLlm(Model),
    HttpsOutcall(Backend),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub provider: Provider,
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

    pub async fn complete(&self, prompt: &str, max_tokens: u32) -> Result<LlmResponse, String> {
        use ic_llm_interface::{CanisterMetrics, InferenceRequest, InferenceResponse, LlmCanister};

        let request = InferenceRequest {
            prompt: prompt.to_string(),
            max_tokens: Some(max_tokens.min(1000)),
            temperature: Some(0.7),
            top_p: None,
            stop_tokens: None,
        };

        let llm_canister = LlmCanister::new();
        let _metrics: CanisterMetrics = llm_canister.metrics().await.map_err(|e| e.to_string())?;

        let response: InferenceResponse = llm_canister
            .infer(request, None)
            .await
            .map_err(|e| e.to_string())?;

        Ok(LlmResponse {
            content: response.inference_result.content,
            provider: Provider::IcLlm(self.model.clone()),
            model: format!("{:?}", self.model),
            tokens_used: response.inference_result.tokens_used.unwrap_or(0) as usize,
        })
    }

    pub fn max_output_tokens(&self) -> u32 {
        match self.model {
            Model::Llama3_1_8B | Model::Llama4Scout => 1000,
            Model::Qwen3_32B => 1000,
        }
    }

    pub fn name(&self) -> &str {
        match self.model {
            Model::Llama3_1_8B => "ic-llm/Llama3.1-8B",
            Model::Qwen3_32B => "ic-llm/Qwen3-32B",
            Model::Llama4Scout => "ic-llm/Llama4-Scout",
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
        messages: &[outcall::AnthropicMessage],
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
                    provider: Provider::HttpsOutcall(Backend::Anthropic),
                    model: "claude-sonnet-4-20250514".to_string(),
                    tokens_used: tokens,
                })
            }
            Backend::OpenAI => {
                let openai_messages: Vec<outcall::OpenAiMessage> = messages
                    .iter()
                    .map(|m| outcall::OpenAiMessage {
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
                    provider: Provider::HttpsOutcall(Backend::OpenAI),
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

pub fn select_provider_for_task(task: &Task) -> Provider {
    if task.complexity == Complexity::Low && task.token_estimate < 800 {
        Provider::IcLlm(Model::Llama3_1_8B)
    } else {
        Provider::HttpsOutcall(Backend::Anthropic)
    }
}

pub async fn execute_with_provider(
    provider: &Provider,
    messages: &[outcall::AnthropicMessage],
    system: Option<&str>,
    max_tokens: u32,
    principal: &str,
    session: &str,
) -> Result<LlmResponse, String> {
    match provider {
        Provider::IcLlm(model) => {
            let provider_impl = IcLlmProvider::new(model.clone());
            let prompt = messages
                .iter()
                .map(|m| format!("{}: {}", m.role, m.content))
                .collect::<Vec<_>>()
                .join("\n");
            let full_prompt = if let Some(sys) = system {
                format!("{}\n\n{}", sys, prompt)
            } else {
                prompt
            };
            provider_impl.complete(&full_prompt, max_tokens).await
        }
        Provider::HttpsOutcall(backend) => {
            let api_key = match backend {
                Backend::Anthropic => std::env::var("ANTHROPIC_API_KEY").ok(),
                Backend::OpenAI => std::env::var("OPENAI_API_KEY").ok(),
            };
            let provider_impl = HttpsOutcallProvider::new(backend.clone(), api_key);
            provider_impl
                .complete(messages, system, max_tokens, principal, session)
                .await
        }
    }
}
