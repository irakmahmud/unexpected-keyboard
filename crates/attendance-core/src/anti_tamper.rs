use attendance_common::{AppError, Result, TimeHelper};

pub struct AntiTamperService;

impl AntiTamperService {
    /// Verify punch GPS geofencing radius (in meters) from office center
    pub fn verify_geofence(
        punch_lat: Option<f64>,
        punch_lon: Option<f64>,
        office_lat: f64,
        office_lon: f64,
        allowed_radius_meters: f64,
    ) -> Result<bool> {
        let (p_lat, p_lon) = match (punch_lat, punch_lon) {
            (Some(lat), Some(lon)) => (lat, lon),
            _ => return Ok(true), // If location not provided by device, allow or flag
        };

        // Haversine formula
        let d_lat = (office_lat - p_lat).to_radians();
        let d_lon = (office_lon - p_lon).to_radians();
        let a = (d_lat / 2.0).sin().powi(2)
            + p_lat.to_radians().cos() * office_lat.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let distance_meters = 6371000.0 * c;

        if distance_meters > allowed_radius_meters {
            return Err(AppError::Validation(format!(
                "Geofence violation: Distance {:.1}m exceeds allowed radius {:.1}m",
                distance_meters, allowed_radius_meters
            )));
        }

        Ok(true)
    }

    /// Check for clock manipulation
    pub fn verify_clock_integrity(client_timestamp: Option<&str>, max_drift_secs: i64) -> Result<()> {
        let res = TimeHelper::validate_client_punch_time(client_timestamp, max_drift_secs);
        if !res.is_valid {
            return Err(AppError::Validation(res.warning.unwrap_or_else(|| "Clock drift violation".to_string())));
        }
        Ok(())
    }
}
