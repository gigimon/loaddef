mod config;
mod dashboard;
mod random_source;
mod stats;

use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_stream::stream;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use clap::Parser;
use serde::Deserialize;
use serde_json::json;
use tokio::time::sleep;
use tracing::info;

use crate::config::Config;
use crate::dashboard::DASHBOARD_HTML;
use crate::random_source::RandomSource;
use crate::stats::{RequestEvent, Stats};

#[derive(Clone)]
struct AppState {
    config: Config,
    random: Arc<RandomSource>,
    stats: Arc<Stats>,
}

#[derive(Debug, Deserialize)]
struct BlobQuery {
    min: Option<usize>,
    max: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct SlowQuery {
    chunks: Option<usize>,
    min_chunk: Option<usize>,
    max_chunk: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "bench_server=info,info".to_string()),
        )
        .init();

    let config = Config::parse();
    if let Err(error) = config.validate() {
        return Err(format!("invalid config: {error}").into());
    }

    let state = AppState {
        random: Arc::new(RandomSource::new(config.seed)),
        stats: Arc::new(Stats::new()),
        config,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/healthz", get(healthz))
        .route("/ok", get(ok_endpoint))
        .route("/e404", get(e404_endpoint))
        .route("/e500", get(e500_endpoint))
        .route("/slow", get(slow_endpoint))
        .route("/blob", get(blob_endpoint))
        .route("/api/stats/summary", get(stats_summary))
        .route("/api/stats/timeseries", get(stats_timeseries))
        .route("/api/stats/reset", post(stats_reset))
        .with_state(state.clone());

    let bind_addr = state.config.bind_addr();
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("bench-server listening on http://{bind_addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn index() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn healthz() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn ok_endpoint(State(state): State<AppState>) -> Response {
    let started = Instant::now();
    let payload = json!({ "ok": true, "message": "benchmark target" });
    json_response_with_tracking(&state, started, "/ok", StatusCode::OK, payload)
}

async fn e404_endpoint(State(state): State<AppState>) -> Response {
    let started = Instant::now();
    let payload = json!({ "error": "not_found" });
    json_response_with_tracking(&state, started, "/e404", StatusCode::NOT_FOUND, payload)
}

async fn e500_endpoint(State(state): State<AppState>) -> Response {
    let started = Instant::now();
    let payload = json!({ "error": "internal_error" });
    json_response_with_tracking(
        &state,
        started,
        "/e500",
        StatusCode::INTERNAL_SERVER_ERROR,
        payload,
    )
}

async fn blob_endpoint(State(state): State<AppState>, Query(query): Query<BlobQuery>) -> Response {
    let started = Instant::now();

    let min = query.min.unwrap_or(state.config.blob_min_bytes);
    let max = query.max.unwrap_or(state.config.blob_max_bytes);

    if min == 0 || min > max {
        let payload = json!({ "error": "invalid_range", "hint": "min must be > 0 and <= max" });
        return json_response_with_tracking(
            &state,
            started,
            "/blob",
            StatusCode::BAD_REQUEST,
            payload,
        );
    }

    let size = state.random.gen_usize_inclusive(min, max);
    let mut data = vec![0_u8; size];
    state.random.fill_bytes(&mut data);

    track_request(
        &state,
        RequestEvent::new(
            "/blob",
            StatusCode::OK.as_u16(),
            size as u64,
            started.elapsed(),
            Instant::now(),
        ),
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header("x-payload-bytes", size.to_string())
        .body(Body::from(data))
        .expect("failed to build /blob response")
}

async fn slow_endpoint(State(state): State<AppState>, Query(query): Query<SlowQuery>) -> Response {
    let started = Instant::now();

    let chunks = query.chunks.unwrap_or(state.config.slow_default_chunks);
    let min_chunk = query.min_chunk.unwrap_or(state.config.slow_min_chunk_bytes);
    let max_chunk = query.max_chunk.unwrap_or(state.config.slow_max_chunk_bytes);

    if chunks == 0 || min_chunk == 0 || min_chunk > max_chunk {
        let payload = json!({
            "error": "invalid_slow_params",
            "hint": "chunks > 0, min_chunk > 0, min_chunk <= max_chunk"
        });
        return json_response_with_tracking(
            &state,
            started,
            "/slow",
            StatusCode::BAD_REQUEST,
            payload,
        );
    }

    let mut planned: Vec<(u64, Bytes)> = Vec::with_capacity(chunks);
    let mut total_bytes: u64 = 0;
    let mut planned_delay_ms: u64 = 0;

    for _ in 0..chunks {
        let delay_ms = state.random.gen_u64_inclusive(
            state.config.slow_min_delay_ms,
            state.config.slow_max_delay_ms,
        );
        let chunk_size = state.random.gen_usize_inclusive(min_chunk, max_chunk);

        let mut chunk = vec![0_u8; chunk_size];
        state.random.fill_bytes(&mut chunk);

        planned_delay_ms += delay_ms;
        total_bytes += chunk_size as u64;
        planned.push((delay_ms, Bytes::from(chunk)));
    }

    // For streaming endpoints, we include planned sleep time so latency metrics reflect slow behavior.
    let simulated_latency = started.elapsed() + Duration::from_millis(planned_delay_ms);
    let simulated_completed_at = Instant::now() + Duration::from_millis(planned_delay_ms);

    track_request(
        &state,
        RequestEvent::new(
            "/slow",
            StatusCode::OK.as_u16(),
            total_bytes,
            simulated_latency,
            simulated_completed_at,
        ),
    );

    let stream = stream! {
        for (delay_ms, chunk) in planned {
            sleep(Duration::from_millis(delay_ms)).await;
            yield Ok::<Bytes, Infallible>(chunk);
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header("x-total-bytes", total_bytes.to_string())
        .header("x-total-delay-ms", planned_delay_ms.to_string())
        .body(Body::from_stream(stream))
        .expect("failed to build /slow response")
}

async fn stats_summary(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.stats.summary())
}

async fn stats_timeseries(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.stats.timeseries())
}

async fn stats_reset(State(state): State<AppState>) -> impl IntoResponse {
    state.stats.reset();
    Json(json!({ "ok": true }))
}

fn track_request(state: &AppState, event: RequestEvent) {
    state.stats.record(event);
}

fn json_response_with_tracking(
    state: &AppState,
    started: Instant,
    endpoint: &'static str,
    status: StatusCode,
    value: serde_json::Value,
) -> Response {
    let body = serde_json::to_vec(&value).expect("failed to encode JSON response");
    let bytes = body.len() as u64;

    track_request(
        state,
        RequestEvent::new(
            endpoint,
            status.as_u16(),
            bytes,
            started.elapsed(),
            Instant::now(),
        ),
    );

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .expect("failed to build JSON response")
}
