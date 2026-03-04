use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "bench-server",
    about = "HTTP benchmark target with in-memory dashboard"
)]
pub struct Config {
    #[arg(long, env = "BENCH_HOST", default_value = "0.0.0.0")]
    pub host: String,

    #[arg(long, env = "BENCH_PORT", default_value_t = 8080)]
    pub port: u16,

    #[arg(long, env = "BENCH_BLOB_MIN_BYTES", default_value_t = 256)]
    pub blob_min_bytes: usize,

    #[arg(long, env = "BENCH_BLOB_MAX_BYTES", default_value_t = 65_536)]
    pub blob_max_bytes: usize,

    #[arg(long, env = "BENCH_SLOW_MIN_DELAY_MS", default_value_t = 100)]
    pub slow_min_delay_ms: u64,

    #[arg(long, env = "BENCH_SLOW_MAX_DELAY_MS", default_value_t = 1_000)]
    pub slow_max_delay_ms: u64,

    #[arg(long, env = "BENCH_SLOW_DEFAULT_CHUNKS", default_value_t = 5)]
    pub slow_default_chunks: usize,

    #[arg(long, env = "BENCH_SLOW_MIN_CHUNK_BYTES", default_value_t = 256)]
    pub slow_min_chunk_bytes: usize,

    #[arg(long, env = "BENCH_SLOW_MAX_CHUNK_BYTES", default_value_t = 4_096)]
    pub slow_max_chunk_bytes: usize,

    #[arg(long, env = "BENCH_SEED")]
    pub seed: Option<u64>,
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.blob_min_bytes == 0 {
            return Err("blob_min_bytes must be > 0".to_string());
        }
        if self.blob_min_bytes > self.blob_max_bytes {
            return Err("blob_min_bytes must be <= blob_max_bytes".to_string());
        }
        if self.slow_min_delay_ms > self.slow_max_delay_ms {
            return Err("slow_min_delay_ms must be <= slow_max_delay_ms".to_string());
        }
        if self.slow_default_chunks == 0 {
            return Err("slow_default_chunks must be > 0".to_string());
        }
        if self.slow_min_chunk_bytes == 0 {
            return Err("slow_min_chunk_bytes must be > 0".to_string());
        }
        if self.slow_min_chunk_bytes > self.slow_max_chunk_bytes {
            return Err("slow_min_chunk_bytes must be <= slow_max_chunk_bytes".to_string());
        }
        Ok(())
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
