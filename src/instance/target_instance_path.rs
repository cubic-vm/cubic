use crate::instance::Instance;

#[derive(Clone)]
pub struct TargetInstancePath {
    pub user: Option<String>,
    pub instance: Option<Instance>,
    pub path: String,
}

impl TargetInstancePath {
    pub fn to_scp(&self) -> String {
        if let Some(instance) = &self.instance {
            format!(
                "scp://{}@127.0.0.1:{}/{}",
                self.user.as_ref().unwrap_or(&instance.user),
                instance.ssh_port,
                self.path
            )
        } else {
            self.path.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::instance::Instance;

    #[test]
    fn test_path() {
        assert_eq!(
            TargetInstancePath {
                user: None,
                instance: None,
                path: "/home/cubic".to_string(),
            }
            .to_scp(),
            "/home/cubic".to_string(),
        );
    }

    #[test]
    fn test_instance_path() {
        assert_eq!(
            TargetInstancePath {
                user: None,
                instance: Some(Instance {
                    user: "testuser".to_string(),
                    ssh_port: 22,
                    ..Instance::default()
                }),
                path: "/home/cubic".to_string(),
            }
            .to_scp(),
            "scp://testuser@127.0.0.1:22//home/cubic"
        )
    }

    #[test]
    fn test_user_instance_path() {
        assert_eq!(
            TargetInstancePath {
                user: Some("cubic".to_string()),
                instance: Some(Instance {
                    user: "testuser".to_string(),
                    ssh_port: 30,
                    ..Instance::default()
                }),
                path: "/home/cubic".to_string(),
            }
            .to_scp(),
            "scp://cubic@127.0.0.1:30//home/cubic"
        )
    }
}
