use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

pub struct Bench {
    message: Option<String>,
    start: Instant,
}

impl Bench {
    pub fn start_silent() -> Self {
        Bench {
            message: None,
            start: Instant::now(),
        }
    }

    pub fn start(message: &str) -> Self {
        Bench {
            message: Some(String::from(message)),
            start: Instant::now(),
        }
    }

    pub fn end(&self) -> () {
        let duration = self.start.elapsed();

        let time = get_time_from_duration(duration);

        let message = self.message.as_deref().unwrap_or("Unnamed Task");
        println!(
            "[{}] {} is complete ({})",
            get_current_time(),
            message,
            time.trim(),
        )
    }

    pub fn end_with(&self, message: &str) -> () {
        let duration = self.start.elapsed();

        let time = get_time_from_duration(duration);

        println!("[{}] {} ({})", get_current_time(), message, time.trim(),)
    }
}

fn get_time_from_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let milliseconds = duration.subsec_millis();

    let mut time = String::from("");

    if hours > 0 {
        time += format!("{}h ", hours).as_str()
    }
    if minutes > 0 {
        time += format!("{}m ", minutes).as_str()
    }
    if seconds > 0 {
        time += format!("{}s ", seconds).as_str()
    }
    if milliseconds > 0 {
        time += format!("{}ms ", milliseconds).as_str()
    }
    if time.is_empty() {
        time += "0ms";
    }

    time
}

fn get_current_time() -> String {
    Utc::now().format("%d.%m.%Y %H:%M:%S").to_string()
}
