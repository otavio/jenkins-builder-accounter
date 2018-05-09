use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone)]
pub(crate) struct Info {
    pub job: String,
    pub number: u32,
    pub duration: Duration,
    pub timestamp: DateTime<Utc>,
}
