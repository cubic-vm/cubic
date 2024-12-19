use crate::commands::Verbosity;
use crate::error::Error;
use crate::qemu::{Qmp, QmpMessage};
use std::thread;
use std::time::Duration;

pub struct Monitor {
    path: String,
    qmp: Qmp,
}

impl Monitor {
    pub fn new(path: &str) -> Result<Self, Error> {
        let mut monitor = Monitor {
            path: path.to_string(),
            qmp: Qmp::new(&format!("{path}/monitor.socket"), Verbosity::Normal)?,
        };
        monitor.init()?;
        Ok(monitor)
    }

    pub fn init(&mut self) -> Result<(), Error> {
        self.qmp.recv().map(|_| ())?;
        self.qmp.execute("qmp_capabilities").map(|_| ())
    }

    pub fn add_unix_socket_chardev(&mut self, id: &str) -> Result<(), Error> {
        self.qmp
            .execute_with_args(
                "chardev-add",
                serde_json::json!({
                    "id": id,
                    "backend": {
                        "type": "socket",
                        "data": {
                            "addr" : {
                                "type" : "unix",
                                "data" : { "path": format!("{}/{id}.socket", self.path) }
                            },
                            "server": true,
                            "wait": false
                        }
                    }
                }),
            )
            .map(|_| ())
    }

    pub fn delete_chardev(&mut self, id: &str) -> Result<(), Error> {
        for _ in 0..10 {
            let result = self
                .qmp
                .execute_with_args("chardev-remove", serde_json::json!({"id": id }));

            if let Result::Ok(QmpMessage::Success { .. }) = result {
                break;
            }

            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    pub fn add_virtserialport_device(&mut self, name: &str, chardev_id: &str) -> Result<(), Error> {
        self.qmp
            .execute_with_args(
                "device_add",
                serde_json::json!({
                    "id": name,
                    "driver": "virtserialport",
                    "bus": "sh_serial.0",
                    "chardev": chardev_id,
                    "name": name
                }),
            )
            .map(|_| ())
    }

    pub fn delete_device(&mut self, id: &str) -> Result<(), Error> {
        self.qmp
            .execute_with_args("device_del", serde_json::json!({"id": id }))
            .map(|_| ())
    }

    pub fn shutdown(&mut self) -> Result<(), Error> {
        self.qmp.execute("system_powerdown")
    }
}
