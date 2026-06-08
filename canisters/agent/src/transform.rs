use ic_cdk::api::management_canister::http_request::HttpResponse;

pub fn transform(response: HttpResponse) -> HttpResponse {
    let non_deterministic = [
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

    let filtered_headers: Vec<_> = response
        .headers
        .into_iter()
        .filter(|header| {
            let key_lower = header.name.to_lowercase();
            !non_deterministic.contains(&key_lower.as_str())
        })
        .collect();

    HttpResponse {
        status: response.status,
        headers: filtered_headers,
        body: response.body,
    }
}