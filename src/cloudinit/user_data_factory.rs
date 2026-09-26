use crate::models::Instance;

pub struct UserDataFactory;

impl UserDataFactory {
    pub fn create(&self, instance: &Instance, pubkey: &str) -> String {
        let user = &instance.user;
        let shell = instance.get_shell();
        let privilege_tool = match instance.get_privilege_tool() {
            "doas" => format!("doas: [permit nopass {user}]"),
            _ => "sudo: ALL=(ALL) NOPASSWD:ALL".to_string(),
        };
        let execute = instance
            .execute
            .as_ref()
            .map(|execute| {
                format!(
                    "runcmd:\n\u{20}\u{20}- \"{}\"\n",
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
            \u{20}\u{20}\u{20}\u{20}hashed_passwd: \"*\"\n\
            \u{20}\u{20}\u{20}\u{20}ssh_authorized_keys: [{pubkey}]\n\
            \u{20}\u{20}\u{20}\u{20}shell: {shell}\n\
            \u{20}\u{20}\u{20}\u{20}{privilege_tool}\n\
            resize_rootfs: noblock\n\
            ssh_genkeytypes: [ed25519]\n\
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
    use crate::models::UserName;
    use std::str::FromStr;

    fn build_instance(os: Option<&str>, execute: Option<&str>) -> Instance {
        Instance {
            user: UserName::from_str("tux").unwrap(),
            os: os.map(str::to_string),
            execute: execute.map(str::to_string),
            ..Instance::default()
        }
    }

    #[test]
    fn test_write_user_data_without_execute() {
        let instance = build_instance(None, None);
        let actual = UserDataFactory.create(&instance, "pubkey");
        let expected = r#"#cloud-config
users:
  - name: tux
    lock_passwd: false
    hashed_passwd: "*"
    ssh_authorized_keys: [pubkey]
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
resize_rootfs: noblock
ssh_genkeytypes: [ed25519]
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
    fn test_write_user_data_for_alpine_with_execute() {
        let instance = build_instance(Some("alpine:3.22"), Some("\"doas apk add vim\""));
        let actual = UserDataFactory.create(&instance, "pubkey");
        let expected = r#"#cloud-config
users:
  - name: tux
    lock_passwd: false
    hashed_passwd: "*"
    ssh_authorized_keys: [pubkey]
    shell: /bin/ash
    doas: [permit nopass tux]
resize_rootfs: noblock
ssh_genkeytypes: [ed25519]
write_files:
  - path: /etc/ssh/sshd_config.d/10-cubic.conf
    content: "AcceptEnv *\n"
runcmd:
  - "\"doas apk add vim\""
"#;
        assert_eq!(
            actual, expected,
            "\nActual: {actual}\nExpected: {expected}\n"
        )
    }

    #[test]
    fn test_write_user_data_escapes_execute() {
        let instance = build_instance(None, Some("a\\b\t\"c\"\nd\re"));
        let actual = UserDataFactory.create(&instance, "pubkey");

        let expected_runcmd = r#"runcmd:
  - "a\\b\t\"c\"\nd\re"
"#;
        assert!(
            actual.ends_with(expected_runcmd),
            "\nActual: {actual}\nExpected suffix: {expected_runcmd}\n"
        )
    }
}
