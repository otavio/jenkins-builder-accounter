use failure::Error;
use regex::RegexSet;
use serde_yaml;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize, PartialOrd, Ord, Eq, PartialEq, Clone)]
pub struct Customer {
    pub name: String,
    job_pattern: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Set {
    customers: Vec<Customer>,
}

impl Set {
    pub fn load(path: &str) -> Result<Self, Error> {
        let mut content = String::new();
        File::open(path)?.read_to_string(&mut content)?;

        Ok(serde_yaml::from_str(&content)?)
    }

    pub fn get(&self, n: usize) -> Option<&Customer> {
        self.customers.get(n)
    }

    pub fn job_patterns(&self) -> Result<RegexSet, Error> {
        Ok(RegexSet::new(
            self.customers
                .iter()
                .map(|c| c.job_pattern.clone())
                .collect::<Vec<_>>(),
        )?)
    }
}
