use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    // Key -> (last_replenished, tokens)
    buckets: Arc<Mutex<HashMap<String, (Instant, f64)>>>,
    max_tokens: f64,
    refill_rate_per_sec: f64,
}

impl RateLimiter {
    pub fn new(max_tokens: f64, refill_rate_per_sec: f64) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            max_tokens,
            refill_rate_per_sec,
        }
    }

    /// Try acquire token for key. Returns true if request is allowed, false if rate limited.
    pub fn check_allowed(&self, key: &str) -> bool {
        let mut map = self.buckets.lock();
        let now = Instant::now();

        let entry = map.entry(key.to_string()).or_insert((now, self.max_tokens));
        let elapsed = now.duration_since(entry.0).as_secs_f64();
        entry.0 = now;

        // Refill tokens based on elapsed time
        entry.1 = (entry.1 + elapsed * self.refill_rate_per_sec).min(self.max_tokens);

        if entry.1 >= 1.0 {
            entry.1 -= 1.0;
            true
        } else {
            false
        }
    }

    /// Clean up idle entries older than max duration
    pub fn cleanup(&self, older_than: Duration) {
        let mut map = self.buckets.lock();
        let now = Instant::now();
        map.retain(|_, (last_time, _)| now.duration_since(*last_time) < older_than);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(2.0, 1.0);
        assert!(limiter.check_allowed("ip1"));
        assert!(limiter.check_allowed("ip1"));
        assert!(!limiter.check_allowed("ip1")); // Exceeded max 2 tokens
        assert!(limiter.check_allowed("ip2")); // Different key allowed
    }
}
