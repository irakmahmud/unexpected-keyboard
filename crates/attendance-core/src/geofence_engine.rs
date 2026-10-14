use attendance_common::{AppError, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeofencePolygon {
    pub name: String,
    pub vertices: Vec<GeoPoint>,
}

pub struct GeofenceEngine;

impl GeofenceEngine {
    /// Haversine great-circle distance between two GPS coordinates in meters
    pub fn haversine_distance_meters(p1: &GeoPoint, p2: &GeoPoint) -> f64 {
        let r = 6_371_000.0; // Earth radius in meters
        let lat1_rad = p1.latitude.to_radians();
        let lat2_rad = p2.latitude.to_radians();
        let d_lat = (p2.latitude - p1.latitude).to_radians();
        let d_lon = (p2.longitude - p1.longitude).to_radians();

        let a = (d_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }

    /// Point-in-polygon test using Ray Casting algorithm (Jordan Curve Theorem)
    pub fn is_point_inside_polygon(point: &GeoPoint, polygon: &[GeoPoint]) -> bool {
        if polygon.len() < 3 {
            return false;
        }

        let mut inside = false;
        let mut j = polygon.len() - 1;

        for i in 0..polygon.len() {
            let pi = &polygon[i];
            let pj = &polygon[j];

            if ((pi.latitude > point.latitude) != (pj.latitude > point.latitude))
                && (point.longitude
                    < (pj.longitude - pi.longitude) * (point.latitude - pi.latitude)
                        / (pj.latitude - pi.latitude)
                        + pi.longitude)
            {
                inside = !inside;
            }
            j = i;
        }

        inside
    }

    /// Detect GPS spoofing / impossible velocity jump between two punches
    /// If an employee punches in location A and 5 minutes later at location B 100km away,
    /// velocity exceeds commercial aircraft speed (e.g. > 900 km/h) = GPS spoofing!
    pub fn verify_velocity_jump(
        prev_point: &GeoPoint,
        prev_time_utc: &str,
        new_point: &GeoPoint,
        new_time_utc: &str,
        max_speed_kmh: f64,
    ) -> Result<()> {
        let t1 = NaiveDateTime::parse_from_str(prev_time_utc, "%Y-%m-%d %H:%M:%S")
            .map_err(|e| AppError::Validation(format!("Invalid prev_time timestamp: {}", e)))?;
        let t2 = NaiveDateTime::parse_from_str(new_time_utc, "%Y-%m-%d %H:%M:%S")
            .map_err(|e| AppError::Validation(format!("Invalid new_time timestamp: {}", e)))?;

        let duration_secs = (t2 - t1).num_seconds().abs() as f64;
        if duration_secs < 1.0 {
            return Ok(());
        }

        let distance_meters = Self::haversine_distance_meters(prev_point, new_point);
        let speed_mps = distance_meters / duration_secs;
        let speed_kmh = speed_mps * 3.6;

        if speed_kmh > max_speed_kmh {
            return Err(AppError::Validation(format!(
                "GPS spoofing detected: velocity of {:.1} km/h exceeds maximum plausible speed {:.1} km/h (distance: {:.1}m in {:.0}s)",
                speed_kmh, max_speed_kmh, distance_meters, duration_secs
            )));
        }

        Ok(())
    }
}
