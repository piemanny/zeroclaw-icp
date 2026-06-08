use crate::outcall;
use crate::tools::result::ToolResult;
use ic_cdk::api::management_canister::http_request::{CanisterHttpRequestArgument, HttpHeader, HttpMethod};

const COST: u64 = 500_000_000;

pub async fn fetch_url(url: &str, method: &str, body: Option<&str>) -> ToolResult {
    let idempotency_key = outcall::generate_idempotency_key(
        &ic_cdk::api::caller().to_string(),
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

    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        method: http_method,
        headers,
        body: body_bytes,
        max_response_bytes: Some(10240),
        transform: None,
        ..Default::default()
    };

    match ic_cdk::api::management_canister::http_request::http_request(request, 10240u128).await {
        Ok((resp,)) => {
            let body_str = String::from_utf8_lossy(&resp.body).to_string();
            ToolResult::ok("fetch_url", &body_str, COST)
        }
        Err(e) => ToolResult::err("fetch_url", &format!("{:?}", e), 0),
    }
}