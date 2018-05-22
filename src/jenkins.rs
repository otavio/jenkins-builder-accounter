use chrono::{DateTime, Duration, Utc};
use failure::Error;
use std::env;

pub struct Credentials {
    pub username: String,
    pub password: String,
    pub server: String,
}

pub fn load_credentials_from_env() -> Result<Credentials, Error> {
    debug!("Loading credentials from environment");
    Ok(Credentials {
        server: env::var("JENKINS_SERVER")?,
        username: env::var("JENKINS_USER")?,
        password: env::var("JENKINS_PASSWORD")?,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct JobInfo {
    pub name: String,
    pub builds: Vec<BuildInfo>,
}

#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub number: u32,
    pub duration: Duration,
    pub timestamp: DateTime<Utc>,
}
