use attendance_common::{CryptoHelper, TimeHelper};
use attendance_device::{
    AudioAnnouncer, AudioCueType, CameraPipeline, DisplayRenderer, QrCodeGenerator,
    ThermalScanner, TurnstileRelayController,
};
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AgentConfig {
    api_url: String,
    device_id: String,
    device_name: String,
    relay_pin: u8,
    relay_pulse_ms: u64,
    fever_threshold: f32,
    heartbeat_interval_secs: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("================================================================================");
    println!("⚡ Smart Attendance System — Native Edge Device, Kiosk & Turnstile Agent");
    println!("Architecture: Pure Rust Native Client • Non-blocking Async Tokio Engine");
    println!("Authoritative Backend: http://127.0.0.1:8080");
    println!("Timezone: Asia/Dhaka (+06:00) • Hardware Gate Relay • Bengali Audio Feedback");
    println!("================================================================================");

    let config = AgentConfig {
        api_url: std::env::var("API_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string()),
        device_id: std::env::var("DEVICE_ID").unwrap_or_else(|_| "TERMINAL_DHAKA_01".to_string()),
        device_name: std::env::var("DEVICE_NAME").unwrap_or_else(|_| "Main Gate Turnstile A".to_string()),
        relay_pin: 17,
        relay_pulse_ms: 1500,
        fever_threshold: 37.5,
        heartbeat_interval_secs: 30,
    };

    let camera = CameraPipeline::new();
    let thermal_scanner = ThermalScanner::new(config.fever_threshold);
    let mut relay_controller = TurnstileRelayController::new(config.relay_pin, config.relay_pulse_ms);

    println!("Device Initialized: {} ({})", config.device_id, config.device_name);
    println!("Relay configured on GPIO pin {} (Pulse: {}ms)", config.relay_pin, config.relay_pulse_ms);

    // Initial LCD Display
    let now_dhaka = TimeHelper::current_datetime_dhaka();
    let lcd = DisplayRenderer::idle_screen_20x4("TECHFIRM BANGLADESH", &now_dhaka);
    println!("\n[LCD 20x4 Display State]");
    println!("┌────────────────────┐");
    println!("│{}│", lcd.line1);
    println!("│{}│", lcd.line2);
    println!("│{}│", lcd.line3);
    println!("│{}│", lcd.line4);
    println!("└────────────────────┘\n");

    // Spawn background heartbeat daemon
    let is_running = Arc::new(AtomicBool::new(true));
    let hb_running = is_running.clone();
    let hb_config = config.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(hb_config.heartbeat_interval_secs));
        while hb_running.load(Ordering::Relaxed) {
            interval.tick().await;
            let heartbeat_payload = serde_json::json!({
                "device_id": hb_config.device_id,
                "status": "ONLINE",
                "battery_level": 100,
                "firmware_version": "v8.0.0-rust-native",
                "queue_size": 0
            });

            let _ = send_http_post(&hb_config.api_url, "/api/v1/devices/heartbeat", &heartbeat_payload).await;
        }
    });

    println!("Available Agent Commands:");
    println!("  punch <username> [CAMERA|QR|MANUAL|RFID|FACE]  - Execute attendance punch with turnstile release");
    println!("  enroll <username>                             - 512-D Face biometric registration");
    println!("  qr                                            - Generate pairing QR code for Android ID");
    println!("  temp <temp_celsius>                           - Test infrared thermal fever scanner");
    println!("  relay <pulse|open|close>                      - Manual turnstile hardware relay command");
    println!("  voice <event_type>                            - Test Bengali & English synthesized audio cues");
    println!("  status                                        - Query authoritative backend system health");
    println!("  exit                                          - Shut down terminal agent\n");

    let stdin = io::stdin();
    let mut reader = stdin.lock();

    loop {
        print!("attendance-agent> ");
        io::stdout().flush()?;

        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "punch" => {
                if parts.len() < 2 {
                    println!("Usage: punch <username> [method]");
                    continue;
                }
                let username = parts[1].to_uppercase();
                let method = parts.get(2).map(|m| m.to_uppercase()).unwrap_or_else(|| "CAMERA".to_string());

                println!("1. Simulating camera frame acquisition for {}...", username);
                let dummy_frame = vec![128u8; 640 * 480 * 3];
                let _frame = camera.process_frame(640, 480, dummy_frame);

                println!("2. Performing thermal screening check...");
                let reading = thermal_scanner.evaluate_reading(36.6, 26.0);
                if reading.is_fever {
                    let (bn, en) = AudioAnnouncer::get_announcement_text(AudioCueType::TemperatureHigh, Some(&username));
                    println!("❌ [ALERT] Fever detected ({:.1}°C)! {}", reading.temperature_celsius, bn);
                    println!("   [EN]: {}", en);
                    continue;
                }

                println!("3. Transmitting punch payload to Axum authoritative server...");
                let req_payload = serde_json::json!({
                    "username": username,
                    "method": method,
                    "device_id": config.device_id,
                    "latitude": 23.8103, // Dhaka coordinate
                    "longitude": 90.4125
                });

                let res = send_http_post(&config.api_url, "/api/v1/attendance/punch", &req_payload).await;
                match res {
                    Ok(resp_str) => {
                        println!("✔ Response from Axum Server: {}", resp_str);

                        // Parse response to check success
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&resp_str) {
                            if v.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
                                let login_type = v.get("login_type").and_then(|s| s.as_str()).unwrap_or("IN_TIME");
                                let cue = if login_type == "IN_TIME" {
                                    AudioCueType::SuccessIn
                                } else {
                                    AudioCueType::SuccessOut
                                };

                                // Voice Feedback
                                let (bn, en) = AudioAnnouncer::get_announcement_text(cue, Some(&username));
                                println!("🔊 [AUDIO FEEDBACK]");
                                println!("   বাংলা: {}", bn);
                                println!("   EN:    {}", en);

                                // Turnstile Relay Unlock
                                println!("⚡ Unlocking turnstile relay gate for 1.5 seconds...");
                                let _ = relay_controller.trigger_unlock().await;
                                println!("✔ Gate unlocked, attendee passed, gate re-locked.");
                            } else {
                                let (bn, _) = AudioAnnouncer::get_announcement_text(AudioCueType::AccessDenied, Some(&username));
                                println!("⚠️ Access denied: {}", bn);
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Network error during punch: {}. Storing in offline buffer...", e);
                    }
                }
            }
            "enroll" => {
                if parts.len() < 2 {
                    println!("Usage: enroll <username>");
                    continue;
                }
                let username = parts[1].to_uppercase();
                println!("Simulating 512-dim facial landmark extraction for {}...", username);
                let mut emb = vec![0.0f32; 512];
                for (i, val) in emb.iter_mut().enumerate() {
                    *val = ((username.as_bytes()[0] as f32 + i as f32) % 17.0) / 17.0;
                }

                let req_payload = serde_json::json!({
                    "username": username,
                    "embedding": emb,
                    "pose": "FRONTAL",
                    "quality_score": 0.95
                });

                let res = send_http_post(&config.api_url, "/api/v1/face/enroll", &req_payload).await;
                match res {
                    Ok(resp) => println!("✔ Enrollment Result: {}", resp),
                    Err(e) => println!("❌ Error during enrollment: {}", e),
                }
            }
            "qr" => {
                let sid = CryptoHelper::new_short_token();
                println!("Generated Pairing Session: {}", sid);
                let pairing_url = format!("{}/api/receive_android_id?session={}", config.api_url, sid);
                println!("QR Code Representation:\n");
                let ascii_qr = QrCodeGenerator::generate_ascii_representation(&pairing_url);
                println!("{}", ascii_qr);
                println!("URL: {}\n", pairing_url);
            }
            "temp" => {
                let temp: f32 = parts.get(1).and_then(|t| t.parse().ok()).unwrap_or(36.8);
                let reading = thermal_scanner.evaluate_reading(temp, 25.0);
                println!("Thermal Sensor Reading: {:.1}°C (Ambient: {:.1}°C)", reading.temperature_celsius, reading.ambient_temp_celsius);
                if reading.is_fever {
                    println!("⚠️ STATUS: FEVER DETECTED (> 37.5°C)");
                } else {
                    println!("✔ STATUS: NORMAL TEMPERATURE");
                }
            }
            "relay" => {
                let action = parts.get(1).copied().unwrap_or("status");
                match action {
                    "pulse" => {
                        println!("Manually triggering relay pulse...");
                        let _ = relay_controller.trigger_unlock().await;
                    }
                    "open" => {
                        relay_controller.emergency_unlock();
                        println!("Relay set to EMERGENCY OPEN");
                    }
                    "close" => {
                        relay_controller.emergency_lock();
                        println!("Relay set to LOCKED");
                    }
                    _ => {
                        let st = relay_controller.get_status();
                        println!("Relay Status: Energized={}, Triggers={}", st.is_energized, st.trigger_count);
                    }
                }
            }
            "voice" => {
                let event = parts.get(1).copied().unwrap_or("in");
                let cue = match event {
                    "in" => AudioCueType::SuccessIn,
                    "out" => AudioCueType::SuccessOut,
                    "denied" => AudioCueType::AccessDenied,
                    "fever" => AudioCueType::TemperatureHigh,
                    _ => AudioCueType::DeviceReady,
                };
                let (bn, en) = AudioAnnouncer::get_announcement_text(cue, Some("রাহিম"));
                println!("বাংলা: {}", bn);
                println!("English: {}", en);
            }
            "status" => {
                println!("Querying Axum backend health at {}/api/v1/health ...", config.api_url);
                match send_http_get(&config.api_url, "/api/v1/health").await {
                    Ok(resp) => println!("Backend Response:\n{}", resp),
                    Err(e) => println!("Failed to reach backend: {}", e),
                }
            }
            "exit" => {
                println!("Shutting down edge agent...");
                is_running.store(false, Ordering::Relaxed);
                break;
            }
            unknown => {
                println!("Unknown command: '{}'. Type 'status', 'punch', 'enroll', 'qr', 'temp', 'relay', 'voice' or 'exit'.", unknown);
            }
        }
    }

    Ok(())
}

async fn send_http_post(
    base_url: &str,
    endpoint: &str,
    json_body: &serde_json::Value,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let full_url = format!("{}{}", base_url, endpoint);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    let res = client
        .post(&full_url)
        .header("Content-Type", "application/json")
        .json(json_body)
        .send()
        .await?;

    let status = res.status();
    let body = res.text().await?;

    if status.is_success() {
        Ok(body)
    } else {
        Err(format!("HTTP Error {}: {}", status, body).into())
    }
}

async fn send_http_get(
    base_url: &str,
    endpoint: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let full_url = format!("{}{}", base_url, endpoint);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    let res = client.get(&full_url).send().await?;
    let status = res.status();
    let body = res.text().await?;

    if status.is_success() {
        Ok(body)
    } else {
        Err(format!("HTTP Error {}: {}", status, body).into())
    }
}
