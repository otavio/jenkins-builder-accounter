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

use customer::CustomerSet;
use failure::{Error, ResultExt};

fn run() -> Result<(), Error> {
    stderrlog::new().verbosity(1).init()?;

    info!("Starting Jenkins Builder Accounter");

    let customers = CustomerSet::load("config/customers.yml")?;
    let jenkins = jenkins::connect().context("connecting to Jenkins server")?;
    for (customer, jobs) in jenkins::get_jenkins_jobs_for_customers(&jenkins, &customers)? {
        let mut total_duration = 0.;

        println!("\nCustomer: {}", customer.name);
        println!("Continous Integration builds:");
        for job in jobs.iter().filter(|j| !j.builds.is_empty()) {
            let duration = job
                .builds
                .iter()
                .map(|b| b.duration.num_minutes() as f64 / 60.)
                .sum::<f64>();
            println!(
                "   - {} Builds: {}  Duration: {:.2}",
                job.name,
                job.builds.len(),
                duration,
            );
            total_duration += duration;
        }
        println!("Total duration: {}", total_duration);
    }

    Ok(())
}

fn main() {
    if let Err(ref e) = run() {
        error!("{}", e);
        e.causes().skip(1).for_each(|e| error!("  due to: {}", e));

        std::process::exit(1);
    }
}
