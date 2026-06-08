use crate::outcall;
use crate::tools::result::ToolResult;
use crate::transform;

const COST: u64 = 500_000_000;

pub fn fetch_url(url: &str, method: &str, body: Option<&str>) -> ToolResult {
    let response = ic_cdk::block_on(async {
        let idempotency_key = outcall::generate_idempotency_key(
            &ic_cdk::api::caller().to_string(),
            &format!("fetch-{}", ic_cdk::api::time()),
            url,
        );

        let body_bytes = body.map(|b| b.as_bytes().to_vec());

        let headers = vec![
            ic_cdk::api::call::HttpHeader {
                name: "Accept".to_string(),
                value: "application/json".to_string(),
            },
            ic_cdk::api::call::HttpHeader {
                name: "Idempotency-Key".to_string(),
                value: idempotency_key,
            },
        ];

        let request = ic_cdk::api::call::HttpRequest {
            url: url.to_string(),
            method: method.to_string(),
            headers,
            body: body_bytes,
            transform: Some("transform".to_string()),
        };

        ic_cdk::api::call::http_request(request, 10_240)
    });

    match response {
        Ok(resp) => {
            let transformed = transform::transform(resp);
            let body_str = String::from_utf8_lossy(&transformed.body).to_string();
            ToolResult::ok("fetch_url", &body_str, COST)
        }
        Err(e) => ToolResult::err("fetch_url", &format!("{:?}", e), 0),
    }
}
