use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use leaklens_api::{router, router_with_capacity};
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn health_is_public_and_reports_no_secrets() {
    let response = router(None)
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 128 * 1024).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes).unwrap(),
        json!({ "status": "ok" })
    );
}

#[tokio::test]
async fn transparent_fixture_reports_public_funding_and_recipient() {
    let response = router(None)
        .oneshot(
            Request::builder()
                .uri("/api/v1/fixtures")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let value: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 128 * 1024).await.unwrap()).unwrap();
    let fixtures = value["fixtures"].as_array().unwrap();
    let transparent = fixtures.iter().find(|v| v["id"] == "transparent").unwrap();
    let codes: Vec<&str> = transparent["assessment"]["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["code"].as_str())
        .collect();
    assert!(codes.contains(&"LL-001"));
    assert!(codes.contains(&"LL-002"));
    assert_eq!(transparent["source"], "sdk_fixture");
}

#[tokio::test]
async fn user_supplied_facts_are_not_misrepresented_as_sdk_validated() {
    let body = json!({"steps":[{"payments":[{"recipient_kind":"transparent","selected_pool":"transparent","amount_zat":25000}],"has_transparent_inputs":false,"has_shielded_inputs":true,"has_prior_shielded_funding":false,"change_pools":[]}]}).to_string();
    let response = router(None)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/assess")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let value: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 128 * 1024).await.unwrap()).unwrap();
    assert_eq!(value["source"], "user_supplied_facts");
    let codes: Vec<&str> = value["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["code"].as_str())
        .collect();
    assert!(codes.contains(&"LL-001"));
    assert!(codes.contains(&"LL-003"));
    assert_eq!(value["complete"], true);
}

#[tokio::test]
async fn invalid_enums_secret_fields_and_oversized_requests_fail() {
    for body in [
        json!({"steps":[{"payments":[{"recipient_kind":"banana","selected_pool":"orchard","amount_zat":10}],"has_transparent_inputs":false,"has_shielded_inputs":true,"has_prior_shielded_funding":false,"change_pools":[]}]}).to_string(),
        json!({"steps":[],"seed_phrase":"secret"}).to_string(),
        "x".repeat(40 * 1024),
    ] {
        let response = router(None).oneshot(Request::builder().method("POST").uri("/api/v1/assess")
            .header(header::CONTENT_TYPE,"application/json").body(Body::from(body)).unwrap()).await.unwrap();
        assert!(response.status().is_client_error());
    }
}

#[tokio::test]
async fn cors_allows_only_a_configured_origin() {
    for (origin, allowed) in [
        ("https://leaklens.vercel.app", true),
        ("https://attacker.example", false),
    ] {
        let response = router(Some("https://leaklens.vercel.app".into()))
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/api/v1/assess")
                    .header(header::ORIGIN, origin)
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_some(),
            allowed
        );
    }
}

#[tokio::test]
async fn no_remote_signing_or_broadcast_routes_exist() {
    for path in ["/sign", "/broadcast", "/api/v1/sign", "/api/v1/broadcast"] {
        for method in ["GET", "POST"] {
            let response = router(None)
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(path)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "{method} {path} must be absent"
            );
        }
    }
}

#[tokio::test]
async fn expensive_routes_fail_fast_at_concurrency_limit() {
    // Zero capacity simulates all worker permits being held without timing races.
    for (method, path, body) in [
        ("GET", "/api/v1/fixtures", String::new()),
        ("POST", "/api/v1/assess", r#"{"steps":[]}"#.to_string()),
    ] {
        let response = router_with_capacity(None, 0)
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers().get(header::RETRY_AFTER).unwrap(), "1");
    }
}

#[tokio::test]
async fn responses_have_conservative_security_headers() {
    let response = router(None)
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );
    assert_eq!(
        response.headers().get("referrer-policy").unwrap(),
        "no-referrer"
    );
    assert_eq!(
        response.headers().get("content-security-policy").unwrap(),
        "default-src 'none'; frame-ancestors 'none'"
    );
    assert_eq!(response.headers().get("x-frame-options").unwrap(), "DENY");
}
