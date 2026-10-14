use std::net::TcpStream;
use std::time::Duration;

pub struct NetworkMonitor;

impl NetworkMonitor {
    /// Probe internet connection by attempting TCP handshake to reliable DNS endpoints
    pub fn check_internet_connection() -> bool {
        let targets = ["8.8.8.8:53", "1.1.1.1:53", "208.67.222.222:53"];
        for target in targets {
            if TcpStream::connect_timeout(&target.parse().unwrap(), Duration::from_millis(1500)).is_ok() {
                return true;
            }
        }
        false
    }
}
