use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use solid_workbook::console::{
    domain::*,
    exports,
    http::{router, AppState},
    memory::*,
    ports::*,
};
use std::sync::Arc;
use tower::ServiceExt;

fn app() -> Router {
    router(AppState {
        journal: Arc::new(MemoryJournal::default()),
        previews: Arc::new(MemoryPreviews::default()),
        reports: Arc::new(exports::defaults(true)),
    })
}
fn payload() -> Value {
    json!({"identity":"one","project":"Atlas","duration_ms":420,"artifacts":["atlas.bin"]})
}
async fn call(
    app: Router,
    method: &str,
    uri: &str,
    value: Option<Value>,
    key: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder().method(method).uri(uri);
    if let Some(key) = key {
        request = request.header("Idempotency-Key", key);
    }
    let body = if let Some(value) = value {
        request = request.header("Content-Type", "application/json");
        Body::from(value.to_string())
    } else {
        Body::empty()
    };
    let response = app.oneshot(request.body(body).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| json!({"raw":String::from_utf8_lossy(&bytes)})),
    )
}
#[tokio::test]
async fn http_create_replay_conflict_and_lookup() {
    let app = app();
    assert_eq!(
        call(
            app.clone(),
            "POST",
            "/api/builds",
            Some(payload()),
            Some("key")
        )
        .await
        .0,
        StatusCode::CREATED
    );
    assert_eq!(
        call(
            app.clone(),
            "POST",
            "/api/builds",
            Some(payload()),
            Some("key")
        )
        .await
        .0,
        StatusCode::OK
    );
    let mut changed = payload();
    changed["duration_ms"] = json!(999);
    assert_eq!(
        call(
            app.clone(),
            "POST",
            "/api/builds",
            Some(changed),
            Some("key")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(app, "GET", "/api/builds/one", None, None).await.1["candidate"]["duration_ms"],
        420
    );
}
#[tokio::test]
async fn invalid_wire_numbers_and_unknown_fields_never_write() {
    let app = app();
    for duration in [
        json!(-1),
        json!(1.5),
        json!(true),
        json!("420"),
        json!(null),
    ] {
        let mut value = payload();
        value["duration_ms"] = duration;
        assert_eq!(
            call(app.clone(), "POST", "/api/builds", Some(value), Some("key"))
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    let mut value = payload();
    value["extra"] = json!(1);
    assert_eq!(
        call(app.clone(), "POST", "/api/builds", Some(value), Some("key"))
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(app, "GET", "/api/builds", None, None).await.1["builds"],
        json!([])
    );
}
#[tokio::test]
async fn missing_bad_keys_and_invalid_domain_never_write() {
    let app = app();
    for key in [None, Some("a/b"), Some(" ")] {
        assert_eq!(
            call(app.clone(), "POST", "/api/builds", Some(payload()), key)
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    let mut value = payload();
    value["artifacts"] = json!(["../secret"]);
    assert_eq!(
        call(app.clone(), "POST", "/api/builds", Some(value), Some("key"))
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(app, "GET", "/api/builds", None, None).await.1["builds"],
        json!([])
    );
}
#[tokio::test]
async fn paging_and_missing_lookup_are_explicit() {
    let app = app();
    for id in ["one", "two"] {
        let mut value = payload();
        value["identity"] = json!(id);
        call(app.clone(), "POST", "/api/builds", Some(value), Some(id)).await;
    }
    let page = call(app.clone(), "GET", "/api/builds?limit=1", None, None)
        .await
        .1;
    let cursor = page["next_cursor"].as_u64().unwrap();
    assert_eq!(
        call(
            app.clone(),
            "GET",
            &format!("/api/builds?limit=1&after={cursor}"),
            None,
            None
        )
        .await
        .1["builds"][0]["candidate"]["identity"],
        "two"
    );
    for query in ["limit=0", "limit=101", "after=-1"] {
        assert!(call(
            app.clone(),
            "GET",
            &format!("/api/builds?{query}"),
            None,
            None
        )
        .await
        .0
        .is_client_error());
    }
    assert_eq!(
        call(app, "GET", "/api/builds/missing", None, None).await.0,
        StatusCode::NOT_FOUND
    );
}
#[tokio::test]
async fn registry_and_processing_use_real_policy() {
    let app = app();
    assert!(call(app.clone(), "GET", "/api/formats", None, None)
        .await
        .1
        .as_array()
        .unwrap()
        .contains(&json!("markdown")));
    call(
        app.clone(),
        "POST",
        "/api/builds",
        Some(payload()),
        Some("key"),
    )
    .await;
    assert_eq!(
        call(app.clone(), "GET", "/api/reports/json", None, None)
            .await
            .1[0]["candidate"]["identity"],
        "one"
    );
    assert_eq!(
        call(app.clone(), "POST", "/api/releases/process", None, None)
            .await
            .1["outcome"],
        "confirmed"
    );
    assert_eq!(
        call(app.clone(), "POST", "/api/releases/process", None, None)
            .await
            .1["outcome"],
        "empty"
    );
    assert_eq!(
        call(app, "GET", "/api/reports/unknown", None, None).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}
#[tokio::test]
async fn lab_does_not_populate_user_journal() {
    let app = app();
    let result = call(app.clone(), "POST", "/api/lab/preview_after", None, None).await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(result.1["preview_effects"], 1);
    assert_eq!(
        call(app, "GET", "/api/builds", None, None).await.1["builds"],
        json!([])
    );
}
#[tokio::test]
async fn storage_failure_is_sanitised_and_unconfirmed() {
    struct Broken;
    impl BuildReader for Broken {
        fn page(&self, _: u64, _: usize) -> Result<Page, StoreError> {
            Err(StoreError)
        }
        fn get(&self, _: &str) -> Result<Option<BuildRecord>, StoreError> {
            Err(StoreError)
        }
    }
    impl BuildWriter for Broken {
        fn record(&self, _: &Candidate, _: &str) -> Result<RecordResult, StoreError> {
            Err(StoreError)
        }
    }
    impl ReleaseQueue for Broken {
        fn pending(&self) -> Result<Option<ReleaseEvent>, StoreError> {
            Err(StoreError)
        }
        fn acknowledge(&self, _: &str, _: &Receipt) -> Result<(), StoreError> {
            Err(StoreError)
        }
    }
    let app = router(AppState {
        journal: Arc::new(Broken),
        previews: Arc::new(MemoryPreviews::default()),
        reports: Arc::new(exports::defaults(false)),
    });
    let (status, body) = call(app, "POST", "/api/builds", Some(payload()), Some("key")).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(body["error"].as_str().unwrap().contains("unconfirmed"));
}
#[tokio::test]
async fn request_body_is_bounded() {
    let mut value = payload();
    value["project"] = json!("x".repeat(20_000));
    assert_eq!(
        call(app(), "POST", "/api/builds", Some(value), Some("key"))
            .await
            .0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
}
#[tokio::test]
async fn serves_static_console_and_health() {
    let app = app();
    for uri in ["/", "/app.css", "/app.js", "/api/health", "/api/ready"] {
        assert_eq!(
            call(app.clone(), "GET", uri, None, None).await.0,
            StatusCode::OK
        );
    }
}
