pub mod events;

use std::{env::consts::OS, fs, path::Path, sync::LazyLock};

use serde::Serialize;

pub static METRICS_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);
pub static HELIX_USER_ID: LazyLock<String> = LazyLock::new(|| {
    // read from credentials file
    let home_dir = std::env::var("HOME").unwrap_or("~/".to_string());
    let config_path = &format!("{home_dir}/.helix/credentials");
    let config_path = Path::new(config_path);
    let user_id = match fs::read_to_string(config_path) {
        Ok(config) => {
            for line in config.lines() {
                if let Some((key, value)) = line.split_once("=")
                    && key.to_lowercase() == "helix_user_id"
                {
                    return value.to_string();
                }
            }
            "".to_string()
        }
        Err(_) => "".to_string(),
    };
    user_id
});

pub const METRICS_URL: &str = "https://logs.helix-db.com";

pub struct HelixMetricsClient {}

impl Default for HelixMetricsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HelixMetricsClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_client(&self) -> &'static LazyLock<reqwest::Client> {
        &METRICS_CLIENT
    }

    pub fn send_event<D: Serialize>(&self, event_type: events::EventType, event_data: D) {
        // get OS
        let os = OS.to_string();

        // get user id
        let user_id = Some(HELIX_USER_ID.as_str().to_string());

        let raw_event = events::RawEvent {
            os,
            user_id,
            event_type,
            event_data,
        };

        drop(
            self.get_client()
                .post(METRICS_URL)
                .body(sonic_rs::to_vec(&raw_event).unwrap())
                .send(),
        );
    }
}

#[derive(Debug)]
pub struct MetricError(String);

impl std::fmt::Display for MetricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MetricError {}

impl From<sonic_rs::Error> for MetricError {
    fn from(e: sonic_rs::Error) -> Self {
        MetricError(e.to_string())
    }
}

impl From<reqwest::Error> for MetricError {
    fn from(e: reqwest::Error) -> Self {
        MetricError(e.to_string())
    }
}
