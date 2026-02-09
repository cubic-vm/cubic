use crate::instance::Instance;

#[derive(Clone)]
pub struct TargetInstancePath {
    pub user: Option<String>,
    pub instance: Option<Instance>,
    pub path: String,
}
