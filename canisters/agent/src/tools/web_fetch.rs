use crate::outcall;
use crate::tools::result::ToolResult;
use ic_cdk_management_canister::{HttpHeader, HttpMethod, HttpRequestArgs};

const COST: u64 = 500_000_000;

pub async fn fetch_url(url: &str, method: &str, body: Option<&str>) -> ToolResult {
    let idempotency_key = outcall::generate_idempotency_key(
        &ic_cdk::api::msg_caller().to_string(),
        &format!("fetch-{}", ic_cdk::api::time()),
        url,
    );

    let body_bytes = body.map(|b| b.as_bytes().to_vec());

    let headers = vec![
        HttpHeader {
            name: "Accept".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Idempotency-Key".to_string(),
            value: idempotency_key,
        },
    ];

    let http_method = if method == "GET" {
        HttpMethod::GET
    } else {
        HttpMethod::POST
    };

    let request = HttpRequestArgs {
        url: url.to_string(),
        method: http_method,
        headers,
        body: body_bytes,
        max_response_bytes: Some(10240),
        transform: None,
        is_replicated: Some(false),
    };

    match ic_cdk_management_canister::http_request(&request).await {
        Ok(response) => {
            let body_str = String::from_utf8_lossy(&response.body).to_string();
            ToolResult::ok("fetch_url", &body_str, COST)
        }
        Err(e) => ToolResult::err("fetch_url", &format!("{:?}", e), 0),
    }
}
