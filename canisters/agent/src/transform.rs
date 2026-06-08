use ic_cdk::api::call::HttpResponse;
use std::collections::HashMap;

const NON_DETERMINISTIC_HEADERS: &[&str] = &[
    "x-request-id",
    "x-request-id",
    "date",
    "cf-ray",
    "cf-cache-status",
    "x-ratelimit-limit",
    "x-ratelimit-remaining",
    "x-ratelimit-reset",
    "x-envoy-upstream-service-time",
    "x-forwarded-for",
    "x-real-ip",
    "strict-transport-security",
    "content-length",
    "x-served-by",
    "x-cache",
    "x-edge",
];

pub fn transform(response: HttpResponse) -> HttpResponse {
    let mut filtered_headers: HashMap<String, String> = HashMap::new();

    for (key, value) in response.headers.iter() {
        let key_lower = key.to_lowercase();
        if !NON_DETERMINISTIC_HEADERS.contains(&key_lower.as_str()) {
            filtered_headers.insert(key.clone(), value.clone());
        }
    }

    HttpResponse {
        status: response.status,
        headers: filtered_headers,
        body: response.body,
    }
}
