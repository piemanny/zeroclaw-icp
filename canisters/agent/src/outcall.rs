use crate::transform::transform;
use ic_cdk::api::call::HttpHeader;
use ic_cdk::api::call::HttpRequest;
use ic_cdk::api::call::HttpResponse;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const MAX_RESPONSE_BYTES: u64 = 10_240;
const MINIMUM_CYCLES_BALANCE: u128 = 500_000_000_000;

pub fn generate_idempotency_key(principal: &str, session: &str, message: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(principal.as_bytes());
    hasher.update(session.as_bytes());
    hasher.update(message.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn check_cycles_balance() -> bool {
    let balance = ic_cdk::api::canister_balance128();
    balance >= MINIMUM_CYCLES_BALANCE
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct AnthropicResponse {
    pub content: Vec<AnthropicContentBlock>,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicContentBlock {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, serde::Serialize, serde:: Deserialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    pub max_tokens: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct OpenAiResponse {
    pub choices: Vec<OpenAiChoice>,
    pub usage: OpenAiUsage,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiChoice {
    pub message: OpenAiMessageContent,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiMessageContent {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

pub async fn anthropic_completion(
    api_key: &str,
    model: &str,
    messages: &[AnthropicMessage],
    system: Option<&str>,
    max_tokens: u32,
    principal: &str,
    session: &str,
) -> Result<(String, usize), String> {
    if !check_cycles_balance() {
        return Err("Insufficient cycles balance for HTTPS outcall".to_string());
    }

    let idempotency_key = generate_idempotency_key(principal, session, &serde_json::to_string(messages).unwrap_or_default());

    let body = AnthropicRequest {
        model: model.to_string(),
        max_tokens,
        messages: messages.to_vec(),
        system: system.map(|s| s.to_string()),
    };

    let body_json = serde_json::to_string(&body).map_err(|e| e.to_string())?;

    let headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "x-api-key".to_string(),
            value: api_key.to_string(),
        },
        HttpHeader {
            name: "anthropic-version".to_string(),
            value: "2023-06-01".to_string(),
        },
        HttpHeader {
            name: "Idempotency-Key".to_string(),
            value: idempotency_key,
        },
    ];

    let request = HttpRequest {
        url: ANTHROPIC_API_URL.to_string(),
        method: "POST".to_string(),
        headers,
        body: Some(body_json.into_bytes()),
        transform: Some("transform".to_string()),
    };

    let response: HttpResponse = ic_cdk::api::call::http_request(request, MAX_RESPONSE_BYTES)
        .await
        .map_err(|e| format!("HTTP request failed: {:?}", e))?;

    let transformed: HttpResponse = transform(response);

    let parsed: AnthropicResponse = serde_json::from_slice(&transformed.body)
        .map_err(|e| format!("Failed to parse response: {} - body: {:?}", e, String::from_utf8_lossy(&transformed.body)))?;

    let content = parsed
        .content
        .first()
        .map(|c| c.text.clone())
        .unwrap_or_default();

    let total_tokens = (parsed.usage.input_tokens + parsed.usage.output_tokens) as usize;

    Ok((content, total_tokens))
}

pub async fn openai_completion(
    api_key: &str,
    model: &str,
    messages: &[OpenAiMessage],
    max_tokens: u32,
    principal: &str,
    session: &str,
) -> Result<(String, usize), String> {
    if !check_cycles_balance() {
        return Err("Insufficient cycles balance for HTTPS outcall".to_string());
    }

    let idempotency_key = generate_idempotency_key(principal, session, &serde_json::to_string(messages).unwrap_or_default());

    let body = OpenAiRequest {
        model: model.to_string(),
        messages: messages.to_vec(),
        max_tokens,
    };

    let body_json = serde_json::to_string(&body).map_err(|e| e.to_string())?;

    let headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", api_key),
        },
        HttpHeader {
            name: "Idempotency-Key".to_string(),
            value: idempotency_key,
        },
    ];

    let request = HttpRequest {
        url: OPENAI_API_URL.to_string(),
        method: "POST".to_string(),
        headers,
        body: Some(body_json.into_bytes()),
        transform: Some("transform".to_string()),
    };

    let response: HttpResponse = ic_cdk::api::call::http_request(request, MAX_RESPONSE_BYTES)
        .await
        .map_err(|e| format!("HTTP request failed: {:?}", e))?;

    let transformed: HttpResponse = transform(response);

    let parsed: OpenAiResponse = serde_json::from_slice(&transformed.body)
        .map_err(|e| format!("Failed to parse response: {} - body: {:?}", e, String::from_utf8_lossy(&transformed.body)))?;

    let content = parsed
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    let total_tokens = (parsed.usage.prompt_tokens + parsed.usage.completion_tokens) as usize;

    Ok((content, total_tokens))
}

pub fn estimate_cost(tokens: usize, provider: &str) -> u64 {
    match provider {
        "anthropic" => (490_000_000u64).saturating_add((tokens as u64).saturating_mul(100_000)),
        "openai" => (450_000_000u64).saturating_add((tokens as u64).saturating_mul(100_000)),
        _ => 500_000_000,
    }
}
