use chrono::{DateTime, Duration, TimeZone, Utc};
use failure::{Error, ResultExt};
use jenkins_api::{Jenkins, JenkinsBuilder};
use std::collections::BTreeMap;
use std::env;

use customer::{Customer, CustomerSet};

struct Credentials {
    username: String,
    password: String,
    server: String,
}

fn load_credentials_from_env() -> Result<Credentials, Error> {
    debug!("Loading credentials from environment");
    Ok(Credentials {
        server: env::var("JENKINS_SERVER").context("read JENKINS_SERVER variable")?,
        username: env::var("JENKINS_USER").context("read JENKINS_USER variable")?,
        password: env::var("JENKINS_PASSWORD").context("read JENKINS_PASSWORD variable")?,
    })
}

pub fn connect() -> Result<Jenkins, Error> {
    let credentials = load_credentials_from_env().context("cannot load credentials")?;
    Ok(JenkinsBuilder::new(&credentials.server)
        .with_user(&credentials.username, Some(&credentials.password))
        .build()
        .context(format!("Fail to connect to server {}", &credentials.server))?)
}

#[derive(Debug, Clone)]
pub struct JobInfo {
    pub name: String,
    pub builds: Vec<BuildInfo>,
}

#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub number: u32,
    pub duration: Duration,
    pub timestamp: DateTime<Utc>,
}

pub fn get_jenkins_jobs_for_customers<'a>(
    jenkins: &Jenkins,
    customers: &'a CustomerSet,
) -> Result<BTreeMap<&'a Customer, Vec<JobInfo>>, Error> {
    let mut customer_use: BTreeMap<&Customer, Vec<JobInfo>> = BTreeMap::new();
    for job in jenkins.get_home()?.jobs {
        let customer_id = customers
            .job_patterns()?
            .matches(&job.name)
            .into_iter()
            .collect::<Vec<_>>()
            .pop();

        if customer_id.is_none() {
            continue;
        }

        let customer = customers.get(customer_id.unwrap()).unwrap();
        let job = job.get_full_job(&jenkins)?;
        let builds = job
            .builds()?
            .into_iter()
            .map(|build| -> Result<BuildInfo, Error> {
                let build = build.get_full_build(&jenkins)?;
                Ok(BuildInfo {
                    number: build.number()?,
                    timestamp: Utc.timestamp((build.timestamp()? / 1000) as i64, 0),
                    duration: {
                        let mut d = Duration::milliseconds(i64::from(build.duration()?));
                        d = d + Duration::minutes(15 - (d.num_minutes() % 15));
                        d
                    },
                })
            });

        let mut builds_info: Vec<BuildInfo> = Vec::new();
        for build in builds {
            match build {
                Ok(b) => {
                    if b.timestamp < Utc::now() - Duration::days(30) {
                        break;
                    }

                    builds_info.push(b)
                }

                Err(e) => {
                    error!("{:?}", e);
                    continue;
                }
            };
        }

        let job = JobInfo {
            name: job.name()?.into(),
            builds: builds_info,
        };

        customer_use
            .entry(customer)
            .and_modify(|e| e.push(job.clone()))
            .or_insert(vec![job]);
    }

    Ok(customer_use)
}
