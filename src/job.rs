use build;

#[derive(Debug, Clone)]
pub(crate) struct Info {
    pub name: String,
    pub builds: Vec<build::Info>,
}
