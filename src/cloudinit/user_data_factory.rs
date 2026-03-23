#[derive(Default)]
pub struct UserDataFactory;

impl UserDataFactory {
    pub fn create(&self, user: &str, pubkey: &str, execute: Option<&str>) -> String {
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
            \u{20}\u{20}\u{20}\u{20}ssh-authorized-keys: [{pubkey}]\n\
            \u{20}\u{20}\u{20}\u{20}shell: /bin/bash\n\
            \u{20}\u{20}\u{20}\u{20}sudo: ALL=(ALL) NOPASSWD:ALL\n\
            {execute}"
        )
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn test_write_user_data_without_execute() {
        let actual = UserDataFactory::default().create("tux", "pubkey", None);
        let expected = r#"#cloud-config
users:
  - name: tux
    lock_passwd: false
    hashed_passwd: $y$j9T$wifmOLBedd7NSaH2IqG4L.$2J.8E.qE57lxapsWosOFod37djHePHg7Go03iDNsRe4
    ssh-authorized-keys: [pubkey]
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
"#;
        assert_eq!(
            actual, expected,
            "\nActual: {actual}\nExpected: {expected}\n"
        )
    }

    #[test]
    fn test_write_user_data_with_execute() {
        let actual =
            UserDataFactory::default().create("tux", "pubkey", Some("\"sudo apt install vim\""));
        let expected = r#"#cloud-config
users:
  - name: tux
    lock_passwd: false
    hashed_passwd: $y$j9T$wifmOLBedd7NSaH2IqG4L.$2J.8E.qE57lxapsWosOFod37djHePHg7Go03iDNsRe4
    ssh-authorized-keys: [pubkey]
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
bootcmd:
  - "\"sudo apt install vim\""
"#;
        assert_eq!(
            actual, expected,
            "\nActual: {actual}\nExpected: {expected}\n"
        )
    }
}
