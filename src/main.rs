extern crate chrono;
#[macro_use]
extern crate log;
extern crate failure;
extern crate jenkins_api;
#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate reqwest;
extern crate serde;
extern crate serde_yaml;
extern crate stderrlog;

mod customer;
mod jenkins;

use customer::CustomerSet;

fn main() -> Result<(), failure::Error> {
    stderrlog::new().verbosity(1).init()?;

    info!("Starting Jenkins Builder Accounter");


    let customers = CustomerSet::load("config/customers.yml")?;
    let jenkins = jenkins::connect()?;
    for (customer, jobs) in jenkins::get_jenkins_jobs_for_customers(&jenkins, &customers)? {
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
