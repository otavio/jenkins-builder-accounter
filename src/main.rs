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

mod build;
mod credentials;
mod customer;

use chrono::{Duration, TimeZone, Utc};
use jenkins_api::JenkinsBuilder;
use std::collections::BTreeMap;

fn main() -> Result<(), failure::Error> {
    stderrlog::new().verbosity(2).init()?;

    info!("Starting Jenkins Builder Accounter");

    let credentials = credentials::load_credentials_from_env()?;
    let jenkins = JenkinsBuilder::new(&credentials.server)
        .with_user(&credentials.username, Some(&credentials.password))
        .build()?;

    let customers = customer::Set::load("config/customers.yml")?;
    let job_patterns = customers.job_patterns()?;

    let mut customer_use: BTreeMap<&customer::Info, Vec<build::Info>> = BTreeMap::new();
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
        let builds = job.builds()?;

        let mut info: Vec<build::Info> = Vec::new();
        for build in builds
            .iter()
            .map(|build| -> Result<build::Info, failure::Error> {
                let build = build.get_full_build(&jenkins)?;
                Ok(build::Info {
                    job: job.name()?.into(),
                    number: build.number()?,
                    timestamp: Utc.timestamp((build.timestamp()? / 1000) as i64, 0),
                    duration: Duration::milliseconds(i64::from(build.duration()?)),
                })
            }) {
            match build {
                Ok(ref build) if (build.timestamp < Utc::now() - Duration::days(30)) => break,
                Ok(build) => info.push(build),
                Err(e) => {
                    error!("{:?}", e);
                    continue;
                }
            };
        }

        customer_use
            .entry(&customer)
            .and_modify(|e| e.append(&mut info))
            .or_insert(info);
    }

    for (customer, builds) in customer_use {
        println!(
            "Customer: {}  Total builds: {}  Total duration: {:#?}",
            customer.name,
            builds.len(),
            builds.iter().map(|b| b.duration.num_seconds()).sum::<i64>() / 3600
        );
    }

    Ok(())
}
