use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const DEFAULT_MIN_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub struct ArxivRateLimiter {
    inner: Arc<Mutex<RateState>>,
    min_interval: Duration,
}

struct RateState {
    last_started: Option<Instant>,
}

impl ArxivRateLimiter {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RateState {
                last_started: None,
            })),
            min_interval,
        }
    }

    pub fn default_gate() -> Self {
        Self::new(DEFAULT_MIN_INTERVAL)
    }

    pub async fn run<F, T>(&self, operation: F) -> T
    where
        F: Future<Output = T>,
    {
        let delay = {
            let guard = self.inner.lock().await;
            guard
                .last_started
                .map(|last| self.min_interval.saturating_sub(last.elapsed()))
                .unwrap_or(Duration::ZERO)
        };
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
        {
            let mut guard = self.inner.lock().await;
            guard.last_started = Some(Instant::now());
        }
        operation.await
    }
}
