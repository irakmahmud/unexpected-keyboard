use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalReading {
    pub temperature_celsius: f32,
    pub is_fever: bool,
    pub ambient_temp_celsius: f32,
    pub measured_at: String,
}

pub struct ThermalScanner {
    fever_threshold_celsius: f32,
}

impl Default for ThermalScanner {
    fn default() -> Self {
        Self {
            fever_threshold_celsius: 37.5,
        }
    }
}

impl ThermalScanner {
    pub fn new(fever_threshold: f32) -> Self {
        Self {
            fever_threshold_celsius: fever_threshold,
        }
    }

    /// Evaluate an infrared body temperature reading
    pub fn evaluate_reading(&self, temp_celsius: f32, ambient_celsius: f32) -> ThermalReading {
        // Temperature compensation based on ambient drift
        let compensated_temp = if ambient_celsius < 18.0 {
            temp_celsius + 0.3
        } else if ambient_celsius > 35.0 {
            temp_celsius - 0.2
        } else {
            temp_celsius
        };

        let is_fever = compensated_temp >= self.fever_threshold_celsius;

        ThermalReading {
            temperature_celsius: compensated_temp,
            is_fever,
            ambient_temp_celsius: ambient_celsius,
            measured_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
