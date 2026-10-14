use attendance_common::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelayTriggerType {
    PulseOpen { duration_ms: u64 },
    HoldOpen,
    HoldClose,
    EmergencyUnlockAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayStatus {
    pub relay_id: u8,
    pub is_energized: bool,
    pub last_triggered_at: String,
    pub trigger_count: u64,
}

pub struct TurnstileRelayController {
    relay_pin: u8,
    pulse_duration: Duration,
    trigger_count: u64,
    is_open: bool,
}

impl TurnstileRelayController {
    pub fn new(relay_pin: u8, pulse_ms: u64) -> Self {
        Self {
            relay_pin,
            pulse_duration: Duration::from_millis(pulse_ms),
            trigger_count: 0,
            is_open: false,
        }
    }

    /// Trigger turnstile gate opening pulse for a valid attendee
    pub async fn trigger_unlock(&mut self) -> Result<()> {
        self.is_open = true;
        self.trigger_count += 1;
        tracing::info!(
            "Turnstile Relay Pin {} PULSE OPEN triggered (Count: {})",
            self.relay_pin,
            self.trigger_count
        );

        // Simulate pulse duration (e.g. 1500ms to allow attendee passage)
        tokio::time::sleep(self.pulse_duration).await;

        self.is_open = false;
        tracing::info!("Turnstile Relay Pin {} CLOSED (Gate re-locked)", self.relay_pin);
        Ok(())
    }

    /// Trigger emergency release (keep open)
    pub fn emergency_unlock(&mut self) {
        self.is_open = true;
        tracing::warn!("EMERGENCY RELAY UNLOCK ACTIVATED ON PIN {}", self.relay_pin);
    }

    /// Lock gate back
    pub fn emergency_lock(&mut self) {
        self.is_open = false;
        tracing::info!("Emergency relay unlock cleared on pin {}", self.relay_pin);
    }

    pub fn get_status(&self) -> RelayStatus {
        RelayStatus {
            relay_id: self.relay_pin,
            is_energized: self.is_open,
            last_triggered_at: chrono::Utc::now().to_rfc3339(),
            trigger_count: self.trigger_count,
        }
    }
}
