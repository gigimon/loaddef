use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use hdrhistogram::Histogram;
use serde::Serialize;

const MAX_LATENCY_US: u64 = 60_000_000;

pub struct RequestEvent {
    pub endpoint: &'static str,
    pub status: u16,
    pub bytes: u64,
    pub latency: Duration,
    pub completed_at: Instant,
}

impl RequestEvent {
    pub fn new(
        endpoint: &'static str,
        status: u16,
        bytes: u64,
        latency: Duration,
        completed_at: Instant,
    ) -> Self {
        Self {
            endpoint,
            status,
            bytes,
            latency,
            completed_at,
        }
    }
}

pub struct Stats {
    clock: RwLock<ClockState>,
    total_requests: AtomicU64,
    total_errors: AtomicU64,
    total_bytes: AtomicU64,
    by_endpoint: DashMap<String, Arc<EndpointCounters>>,
    by_status: DashMap<u16, AtomicU64>,
    timeseries: Mutex<BTreeMap<u64, TimeseriesBucket>>,
}

struct ClockState {
    started_at: SystemTime,
    start_instant: Instant,
}

struct EndpointCounters {
    requests: AtomicU64,
    errors: AtomicU64,
    bytes: AtomicU64,
    latency_us: Mutex<Histogram<u64>>,
}

impl EndpointCounters {
    fn new() -> Self {
        Self {
            requests: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            latency_us: Mutex::new(
                Histogram::<u64>::new_with_bounds(1, MAX_LATENCY_US, 3)
                    .expect("failed to build histogram"),
            ),
        }
    }

    fn record_latency_us(&self, value: u64) {
        let mut hist = self.latency_us.lock().expect("latency histogram poisoned");
        let clamped = value.clamp(1, MAX_LATENCY_US);
        let _ = hist.record(clamped);
    }
}

#[derive(Default)]
struct TimeseriesBucket {
    requests: u64,
    errors: u64,
    bytes: u64,
    by_endpoint: BTreeMap<String, u64>,
}

#[derive(Serialize)]
pub struct SummaryResponse {
    pub started_at_unix_ms: u128,
    pub uptime_seconds: u64,
    pub total_requests: u64,
    pub total_errors: u64,
    pub total_bytes: u64,
    pub avg_rps: f64,
    pub by_endpoint: Vec<EndpointSummary>,
    pub by_status: Vec<StatusCount>,
}

#[derive(Serialize)]
pub struct EndpointSummary {
    pub endpoint: String,
    pub requests: u64,
    pub errors: u64,
    pub bytes: u64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[derive(Serialize)]
pub struct StatusCount {
    pub status: u16,
    pub count: u64,
}

#[derive(Serialize)]
pub struct TimeseriesResponse {
    pub points: Vec<TimeseriesPoint>,
}

#[derive(Serialize)]
pub struct TimeseriesPoint {
    pub second_offset: u64,
    pub requests: u64,
    pub errors: u64,
    pub bytes: u64,
    pub by_endpoint: BTreeMap<String, u64>,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            clock: RwLock::new(ClockState {
                started_at: SystemTime::now(),
                start_instant: Instant::now(),
            }),
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            by_endpoint: DashMap::new(),
            by_status: DashMap::new(),
            timeseries: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn record(&self, event: RequestEvent) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(event.bytes, Ordering::Relaxed);

        let endpoint_name = event.endpoint.to_string();
        let endpoint = self
            .by_endpoint
            .entry(endpoint_name.clone())
            .or_insert_with(|| Arc::new(EndpointCounters::new()))
            .clone();

        endpoint.requests.fetch_add(1, Ordering::Relaxed);
        endpoint.bytes.fetch_add(event.bytes, Ordering::Relaxed);

        if event.status >= 400 {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
            endpoint.errors.fetch_add(1, Ordering::Relaxed);
        }

        let latency_us = event.latency.as_micros().min(u128::from(MAX_LATENCY_US)) as u64;
        endpoint.record_latency_us(latency_us.max(1));

        self.by_status
            .entry(event.status)
            .and_modify(|count| {
                count.fetch_add(1, Ordering::Relaxed);
            })
            .or_insert_with(|| AtomicU64::new(1));

        let second_offset = {
            let clock = self.clock.read().expect("clock lock poisoned");
            event
                .completed_at
                .saturating_duration_since(clock.start_instant)
                .as_secs()
        };

        let mut buckets = self.timeseries.lock().expect("timeseries lock poisoned");
        let bucket = buckets.entry(second_offset).or_default();
        bucket.requests += 1;
        bucket.bytes += event.bytes;
        if event.status >= 400 {
            bucket.errors += 1;
        }
        *bucket.by_endpoint.entry(endpoint_name).or_default() += 1;
    }

    pub fn summary(&self) -> SummaryResponse {
        let (started_at_unix_ms, uptime_seconds, uptime_secs_f64) = {
            let clock = self.clock.read().expect("clock lock poisoned");
            let started_at_unix_ms = clock
                .started_at
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let elapsed = clock.start_instant.elapsed();
            let uptime_seconds = elapsed.as_secs();
            let uptime_secs_f64 = elapsed.as_secs_f64().max(0.001);
            (started_at_unix_ms, uptime_seconds, uptime_secs_f64)
        };

        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let total_errors = self.total_errors.load(Ordering::Relaxed);
        let total_bytes = self.total_bytes.load(Ordering::Relaxed);

        let mut by_endpoint = self
            .by_endpoint
            .iter()
            .map(|entry| {
                let counters = entry.value();
                let hist = counters
                    .latency_us
                    .lock()
                    .expect("latency histogram poisoned");

                let p50_ms = if hist.is_empty() {
                    0.0
                } else {
                    hist.value_at_quantile(0.50) as f64 / 1_000.0
                };
                let p95_ms = if hist.is_empty() {
                    0.0
                } else {
                    hist.value_at_quantile(0.95) as f64 / 1_000.0
                };
                let p99_ms = if hist.is_empty() {
                    0.0
                } else {
                    hist.value_at_quantile(0.99) as f64 / 1_000.0
                };

                EndpointSummary {
                    endpoint: entry.key().clone(),
                    requests: counters.requests.load(Ordering::Relaxed),
                    errors: counters.errors.load(Ordering::Relaxed),
                    bytes: counters.bytes.load(Ordering::Relaxed),
                    p50_ms,
                    p95_ms,
                    p99_ms,
                }
            })
            .collect::<Vec<_>>();

        by_endpoint.sort_by(|a, b| a.endpoint.cmp(&b.endpoint));

        let mut by_status = self
            .by_status
            .iter()
            .map(|entry| StatusCount {
                status: *entry.key(),
                count: entry.value().load(Ordering::Relaxed),
            })
            .collect::<Vec<_>>();
        by_status.sort_by_key(|item| item.status);

        SummaryResponse {
            started_at_unix_ms,
            uptime_seconds,
            total_requests,
            total_errors,
            total_bytes,
            avg_rps: total_requests as f64 / uptime_secs_f64,
            by_endpoint,
            by_status,
        }
    }

    pub fn timeseries(&self) -> TimeseriesResponse {
        let buckets = self.timeseries.lock().expect("timeseries lock poisoned");
        let points = buckets
            .iter()
            .map(|(second_offset, bucket)| TimeseriesPoint {
                second_offset: *second_offset,
                requests: bucket.requests,
                errors: bucket.errors,
                bytes: bucket.bytes,
                by_endpoint: bucket.by_endpoint.clone(),
            })
            .collect::<Vec<_>>();

        TimeseriesResponse { points }
    }

    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.total_errors.store(0, Ordering::Relaxed);
        self.total_bytes.store(0, Ordering::Relaxed);

        self.by_endpoint.clear();
        self.by_status.clear();

        {
            let mut buckets = self.timeseries.lock().expect("timeseries lock poisoned");
            buckets.clear();
        }

        {
            let mut clock = self.clock.write().expect("clock lock poisoned");
            clock.started_at = SystemTime::now();
            clock.start_instant = Instant::now();
        }
    }
}
