extern crate chrono;
#[macro_use]
extern crate log;
extern crate failure;
extern crate jenkins_api;
#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate serde;
extern crate serde_yaml;
extern crate stderrlog;

mod customer;
mod jenkins;

use customer::{Customer, CustomerSet};
use jenkins::{BuildInfo, JobInfo};

use chrono::{Duration, TimeZone, Utc};
use jenkins_api::JenkinsBuilder;
use std::collections::BTreeMap;

fn main() -> Result<(), failure::Error> {
    stderrlog::new().verbosity(1).init()?;

    info!("Starting Jenkins Builder Accounter");

    let credentials = jenkins::load_credentials_from_env()?;
    let jenkins = JenkinsBuilder::new(&credentials.server)
        .with_user(&credentials.username, Some(&credentials.password))
        .build()?;

    let customers = CustomerSet::load("config/customers.yml")?;
    let job_patterns = customers.job_patterns()?;

    let mut customer_use: BTreeMap<&Customer, Vec<JobInfo>> = BTreeMap::new();
    for job in jenkins.get_home()?.jobs {
        let customer_id = job_patterns
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
            .map(|build| -> Result<BuildInfo, failure::Error> {
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
            .entry(&customer)
            .and_modify(|e| e.push(job.clone()))
            .or_insert(vec![job]);
    }

    for (customer, jobs) in customer_use {
        println!("Customer: {}", customer.name);
        for job in jobs {
            if job.builds.is_empty() {
                continue;
            }

            println!(
                " - Job: {}  Total builds: {}  Total duration: {:.2}",
                job.name,
                job.builds.len(),
                job.builds
                    .iter()
                    .map(|b| b.duration.num_minutes() as f64 / 60.0)
                    .sum::<f64>()
            );
        }
    }

    Ok(())
}
