use reqwest::blocking::Client;
use serde::Serialize;

#[derive(Serialize)]
pub struct CompressMessage {
    pub log_type: String,
    pub device_name: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub time_taken: String,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct HealthMessage {
    pub log_type: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct Message {
    message: HealthMessage,
}

#[allow(dead_code)]
pub fn send_compression_log(log_message: CompressMessage) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let response = client
        .post("http://localhost:5140")
        .json(&log_message)
        .send()?;

    if response.status().is_success() {
        println!("Log sent successfully!");
    } else {
        eprintln!("Failed to send log: {:?}", response.status());
    }

    Ok(())
}

#[allow(dead_code)]
pub fn send_health_log(log_message: HealthMessage) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let message = Message {
        message: log_message,
    };

    let response = client
        .post("http://localhost:5140")
        .json(&message)
        .send()?;

    if response.status().is_success() {
        println!("Log sent successfully!");
    } else {
        eprintln!("Failed to send log: {:?}", response.status());
    }

    Ok(())
}
