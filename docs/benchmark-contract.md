# Benchmark Contract

## Scope

This server is a deterministic API surface with stochastic response behavior for load testing tools comparison.
All metrics are in-memory for current process lifetime and resettable via API.

## Endpoints

### Health and UI
- `GET /healthz`
  - Purpose: liveness probe.
  - Response: `200` JSON `{ "status": "ok" }`.
  - Excluded from benchmark scenarios.
- `GET /`
  - Purpose: dashboard UI.
  - Response: `200` HTML.
  - Excluded from benchmark scenarios.

### Load endpoints
- `GET /ok`
  - Response: `200` JSON.
  - Profile usage: baseline throughput.

- `GET /e404`
  - Response: always `404` JSON.
  - Profile usage: controlled client/server error handling.

- `GET /e500`
  - Response: always `500` JSON.
  - Profile usage: failure path and retries behavior.

- `GET /blob?min=<usize>&max=<usize>`
  - Defaults: `min=256`, `max=65536` bytes (from config).
  - Behavior: payload size is uniformly random in `[min, max]`, bytes are freshly generated on each request.
  - Response: `200` `application/octet-stream`.
  - Invalid params: `400` JSON.

- `GET /slow?chunks=<usize>&min_chunk=<usize>&max_chunk=<usize>`
  - Defaults: `chunks=5`, `min_chunk=256`, `max_chunk=4096` (from config).
  - Behavior: response is streamed by chunks; before each chunk, random delay in `[100ms, 1000ms]`.
  - Response: `200` `application/octet-stream` (chunked transfer).
  - Invalid params: `400` JSON.

## Statistics API

- `GET /api/stats/summary`
  - Returns global counters and per-endpoint aggregates:
    - `total_requests`, `total_errors`, `total_bytes`, `avg_rps`
    - per endpoint: `requests`, `errors`, `bytes`, `p50_ms`, `p95_ms`, `p99_ms`
    - status code distribution

- `GET /api/stats/timeseries`
  - Returns second-level points from process start:
    - `requests`, `errors`, `bytes`, `by_endpoint`

- `POST /api/stats/reset`
  - Clears all counters and time series for a new run.

## Comparison Protocol (Tool-Agnostic)

Use identical protocol for each load tool:

1. Warmup: 20-30 seconds (not used in comparison table).
2. Measurement: fixed duration (recommended: 2-5 minutes per profile).
3. Cooldown: 10 seconds.
4. 3 repeated runs per profile.
5. Reset stats before every run via `POST /api/stats/reset`.
6. Keep target host and machine identical between tools.

## Comparable Metrics

Primary metrics:
- Throughput: achieved RPS (tool-side and server-side average).
- Latency: p50, p95, p99.
- Error rate: `4xx + 5xx` share.
- Stability: run-to-run variance on p95/p99 and RPS.

Secondary metrics:
- Bytes/sec.
- CPU and memory of load generator and server.

## Fair Comparison Constraints

- Same hardware for all tools.
- Fixed CPU governor/performance mode.
- No unrelated background jobs.
- Same network path and connection settings (keep-alive, TLS mode, DNS behavior).
- Same request mix and payload boundaries per profile.

## Baseline Profiles

- Profile A: 100% `/ok`
- Profile B: mixed `/ok` + `/e404` + `/e500` + `/blob`
- Profile C: latency stress with high share of `/slow`
- Profile D: spike (step-up concurrency or target RPS)

For each profile, freeze:
- Duration, warmup, cooldown.
- Concurrency/VU and (if used) target RPS.
- Endpoint distribution.
