use crate::models::UserName;

#[derive(Default)]
pub struct UserDataFactory;

impl UserDataFactory {
    pub fn create(&self, user: &UserName, pubkey: &str, execute: Option<&str>) -> String {
        let execute = execute
            .map(|execute| {
                format!(
                    "bootcmd:\n\u{20}\u{20}- \"{}\"\n",
                    execute
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace('\t', "\\t")
                )
            })
            .unwrap_or_default();

        format!(
            "\
            #cloud-config\n\
            users:\n\
            \u{20}\u{20}- name: {user}\n\
            \u{20}\u{20}\u{20}\u{20}lock_passwd: false\n\
            \u{20}\u{20}\u{20}\u{20}hashed_passwd: $y$j9T$wifmOLBedd7NSaH2IqG4L.$2J.8E.qE57lxapsWosOFod37djHePHg7Go03iDNsRe4\n\
            \u{20}\u{20}\u{20}\u{20}ssh_authorized_keys: [{pubkey}]\n\
            \u{20}\u{20}\u{20}\u{20}shell: /bin/bash\n\
            \u{20}\u{20}\u{20}\u{20}sudo: ALL=(ALL) NOPASSWD:ALL\n\
            write_files:\n\
            \u{20}\u{20}- path: /etc/ssh/sshd_config.d/10-cubic.conf\n\
            \u{20}\u{20}\u{20}\u{20}content: \"AcceptEnv *\\n\"\n\
            {execute}"
        )
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;
    use std::str::FromStr;

    #[test]
    fn test_write_user_data_without_execute() {
        let actual =
            UserDataFactory::default().create(&UserName::from_str("tux").unwrap(), "pubkey", None);
        let expected = r#"#cloud-config
users:
  - name: tux
    lock_passwd: false
    hashed_passwd: $y$j9T$wifmOLBedd7NSaH2IqG4L.$2J.8E.qE57lxapsWosOFod37djHePHg7Go03iDNsRe4
    ssh_authorized_keys: [pubkey]
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
write_files:
  - path: /etc/ssh/sshd_config.d/10-cubic.conf
    content: "AcceptEnv *\n"
"#;
        assert_eq!(
            actual, expected,
            "\nActual: {actual}\nExpected: {expected}\n"
        )
    }

    #[test]
    fn test_write_user_data_with_execute() {
        let actual = UserDataFactory::default().create(
            &UserName::from_str("tux").unwrap(),
            "pubkey",
            Some("\"sudo apt install vim\""),
        );
        let expected = r#"#cloud-config
users:
  - name: tux
    lock_passwd: false
    hashed_passwd: $y$j9T$wifmOLBedd7NSaH2IqG4L.$2J.8E.qE57lxapsWosOFod37djHePHg7Go03iDNsRe4
    ssh_authorized_keys: [pubkey]
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
write_files:
  - path: /etc/ssh/sshd_config.d/10-cubic.conf
    content: "AcceptEnv *\n"
bootcmd:
  - "\"sudo apt install vim\""
"#;
        assert_eq!(
            actual, expected,
            "\nActual: {actual}\nExpected: {expected}\n"
        )
    }

    #[test]
    fn test_write_user_data_escapes_execute() {
        let actual = UserDataFactory::default().create(
            &UserName::from_str("tux").unwrap(),
            "pubkey",
            Some("a\\b\t\"c\"\nd\re"),
        );

        let expected_bootcmd = r#"bootcmd:
  - "a\\b\t\"c\"\nd\re"
"#;
        assert!(
            actual.ends_with(expected_bootcmd),
            "\nActual: {actual}\nExpected suffix: {expected_bootcmd}\n"
        )
    }
}
