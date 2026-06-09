use ic_cdk_management_canister::{HttpHeader, HttpMethod, HttpRequestArgs};
use sha2::{Digest, Sha256};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const MAX_RESPONSE_BYTES: u64 = 10_240;
const MINIMUM_CYCLES_BALANCE: u128 = 500_000_000_000;

const CYCLES_PER_HTTP_OUTCALL: u128 = 20_000_000_000;

pub fn generate_idempotency_key(principal: &str, session: &str, message: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(principal.as_bytes());
    hasher.update(session.as_bytes());
    hasher.update(message.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn check_cycles_balance() -> bool {
    let balance = ic_cdk::api::canister_cycle_balance();
    balance >= MINIMUM_CYCLES_BALANCE
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct AnthropicResponse {
    pub content: Vec<AnthropicContentBlock>,
    pub usage: AnthropicUsage,
}

#[derive(Debug, serde::Deserialize)]
pub struct AnthropicContentBlock {
    pub text: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    pub max_tokens: u32,
}

#[derive(Debug, serde::Deserialize)]
pub struct OpenAiResponse {
    pub choices: Vec<OpenAiChoice>,
    pub usage: OpenAiUsage,
}

#[derive(Debug, serde::Deserialize)]
pub struct OpenAiChoice {
    pub message: OpenAiMessageContent,
}

#[derive(Debug, serde::Deserialize)]
pub struct OpenAiMessageContent {
    pub content: String,
}

#[derive(Debug, serde::Deserialize)]
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

    let idempotency_key = generate_idempotency_key(
        principal,
        session,
        &serde_json::to_string(messages).unwrap_or_default(),
    );

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

    let request = HttpRequestArgs {
        url: ANTHROPIC_API_URL.to_string(),
        method: HttpMethod::POST,
        headers,
        body: Some(body_json.into_bytes()),
        max_response_bytes: Some(MAX_RESPONSE_BYTES),
        transform: None,
        is_replicated: Some(false),
    };

    let response = ic_cdk_management_canister::http_request(&request)
        .await
        .map_err(|e| format!("HTTP request failed: {:?}", e))?;

    if response.status != candid::Nat::from(200u32) {
        let status = response.status;
        let body_preview = String::from_utf8_lossy(&response.body).chars().take(200).collect::<String>();
        return Err(format!(
            "HTTP request returned non-200 status: {}. Body: {}",
            status, body_preview
        ));
    }

    let parsed: AnthropicResponse = serde_json::from_slice(&response.body)
        .map_err(|e| {
            format!(
                "Failed to parse response: {} - body: {:?}",
                e,
                String::from_utf8_lossy(&response.body)
            )
        })?;

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

    let idempotency_key = generate_idempotency_key(
        principal,
        session,
        &serde_json::to_string(messages).unwrap_or_default(),
    );

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

    let request = HttpRequestArgs {
        url: OPENAI_API_URL.to_string(),
        method: HttpMethod::POST,
        headers,
        body: Some(body_json.into_bytes()),
        max_response_bytes: Some(MAX_RESPONSE_BYTES),
        transform: None,
        is_replicated: Some(false),
    };

    let response = ic_cdk_management_canister::http_request(&request)
        .await
        .map_err(|e| format!("HTTP request failed: {:?}", e))?;

    if response.status != candid::Nat::from(200u32) {
        let status = response.status;
        let body_preview = String::from_utf8_lossy(&response.body).chars().take(200).collect::<String>();
        return Err(format!(
            "HTTP request returned non-200 status: {}. Body: {}",
            status, body_preview
        ));
    }

    let parsed: OpenAiResponse = serde_json::from_slice(&response.body)
        .map_err(|e| {
            format!(
                "Failed to parse response: {} - body: {:?}",
                e,
                String::from_utf8_lossy(&response.body)
            )
        })?;

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
        "anthropic" => (CYCLES_PER_HTTP_OUTCALL as u64).saturating_add((tokens as u64).saturating_mul(100_000)),
        "openai" => (CYCLES_PER_HTTP_OUTCALL as u64).saturating_add((tokens as u64).saturating_mul(100_000)),
        _ => CYCLES_PER_HTTP_OUTCALL as u64,
    }
}
