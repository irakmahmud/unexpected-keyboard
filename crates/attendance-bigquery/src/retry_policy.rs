use std::time::Duration;

#[derive(Debug, Clone)]
pub struct BackoffPolicy {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub max_retries: u32,
}

impl Default for BackoffPolicy {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            max_retries: 8,
        }
    }
}

impl BackoffPolicy {
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt >= self.max_retries {
            return self.max_delay;
        }

        let delay_ms = (self.initial_delay.as_millis() as f64) * self.multiplier.powi(attempt as i32);
        let capped = Duration::from_millis(delay_ms as u64).min(self.max_delay);
        capped
    }
}
