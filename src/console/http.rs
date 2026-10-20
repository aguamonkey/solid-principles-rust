use super::{domain::*, exports::Reports, lab, policy::*, ports::*};
use axum::{
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub journal: Arc<dyn Journal>,
    pub previews: Arc<dyn PreviewSink + Send + Sync>,
    pub reports: Arc<Reports>,
}
#[derive(Debug)]
pub struct ApiError(StatusCode, String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({"error":self.1}))).into_response()
    }
}
fn storage(_: StoreError) -> ApiError {
    ApiError(
        StatusCode::SERVICE_UNAVAILABLE,
        "Storage unconfirmed; retry the original key and content.".into(),
    )
}
fn invalid(error: impl ToString) -> ApiError {
    ApiError(StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
}
async fn blocking<T: Send + 'static>(
    action: impl FnOnce() -> Result<T, ApiError> + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(action).await.map_err(|_| {
        ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Background operation failed; result may be unconfirmed.".into(),
        )
    })?
}
pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { Html(include_str!("../../console/index.html")) }),
        )
        .route(
            "/app.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css")],
                    include_str!("../../console/app.css"),
                )
            }),
        )
        .route(
            "/app.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript")],
                    include_str!("../../console/app.js"),
                )
            }),
        )
        .route(
            "/api/health",
            get(|| async { Json(serde_json::json!({"status":"ok"})) }),
        )
        .route("/api/ready", get(ready))
        .route("/api/builds", get(list).post(record))
        .route("/api/builds/{identity}", get(lookup))
        .route("/api/formats", get(formats))
        .route("/api/reports/{format}", get(export))
        .route("/api/releases/process", post(process))
        .route("/api/lab/{scenario}", post(run_lab))
        .layer(DefaultBodyLimit::max(16 * 1024))
        .with_state(state)
}
#[derive(Deserialize)]
pub struct Paging {
    #[serde(default)]
    after: u64,
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize {
    25
}
async fn ready(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    blocking(move || {
        state.journal.page(0, 1).map_err(storage)?;
        Ok(Json(serde_json::json!({"status":"ready"})))
    })
    .await
}
async fn list(
    State(state): State<AppState>,
    Query(page): Query<Paging>,
) -> Result<Json<Page>, ApiError> {
    validate_page(page.after, page.limit).map_err(invalid)?;
    blocking(move || {
        state
            .journal
            .page(page.after, page.limit)
            .map(Json)
            .map_err(storage)
    })
    .await
}
async fn lookup(
    State(state): State<AppState>,
    Path(identity): Path<String>,
) -> Result<Json<BuildRecord>, ApiError> {
    blocking(move || {
        state
            .journal
            .get(&identity)
            .map_err(storage)?
            .map(Json)
            .ok_or(ApiError(StatusCode::NOT_FOUND, "Build not found".into()))
    })
    .await
}
async fn record(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CandidateInput>,
) -> Result<Response, ApiError> {
    let key = headers
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| invalid("Idempotency-Key is required"))?
        .to_owned();
    validate_key(&key).map_err(invalid)?;
    let candidate: Candidate = input.try_into().map_err(invalid)?;
    blocking(move || {
        let result = record_build(state.journal.as_ref(), &candidate, &key).map_err(storage)?;
        let status = match result.outcome {
            RecordKind::Created => StatusCode::CREATED,
            RecordKind::Replayed => StatusCode::OK,
            RecordKind::Conflict => StatusCode::CONFLICT,
        };
        Ok((status, Json(result)).into_response())
    })
    .await
}
async fn formats(State(state): State<AppState>) -> Json<Vec<String>> {
    Json(state.reports.formats())
}
async fn export(
    State(state): State<AppState>,
    Path(format): Path<String>,
    Query(page): Query<Paging>,
) -> Result<Response, ApiError> {
    validate_page(page.after, page.limit).map_err(invalid)?;
    if !state.reports.formats().contains(&format) {
        return Err(invalid("Unsupported report format"));
    }
    blocking(move || {
        let (artifact, cursor) = state
            .reports
            .export(state.journal.as_ref(), &format, page.after, page.limit)
            .map_err(storage)?;
        let mut response =
            ([(header::CONTENT_TYPE, artifact.media_type)], artifact.body).into_response();
        if let Some(cursor) = cursor {
            response.headers_mut().insert(
                "X-Next-Cursor",
                cursor.to_string().parse().map_err(invalid)?,
            );
        }
        Ok(response)
    })
    .await
}
async fn process(State(state): State<AppState>) -> Result<Response, ApiError> {
    blocking(move || {
        let result = process_release(state.journal.as_ref(), state.previews.as_ref());
        let status = match result.outcome {
            DeliveryKind::Empty | DeliveryKind::Confirmed => StatusCode::OK,
            _ => StatusCode::SERVICE_UNAVAILABLE,
        };
        Ok((status, Json(result)).into_response())
    })
    .await
}
async fn run_lab(Path(scenario): Path<lab::Scenario>) -> Json<lab::LabResult> {
    Json(lab::run(scenario))
}
