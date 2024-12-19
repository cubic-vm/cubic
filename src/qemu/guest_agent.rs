use crate::commands::Verbosity;
use crate::error::Error;
use crate::qemu::{Qmp, QmpMessage};
use std::thread;
use std::time::Duration;

pub struct GuestAgent {
    qmp: Qmp,
}

impl GuestAgent {
    pub fn new(path: &str) -> Result<Self, Error> {
        Ok(GuestAgent {
            qmp: Qmp::new(path, Verbosity::Normal)?,
        })
    }

    pub fn sync(&mut self) -> Result<(), Error> {
        let arg: serde_json::Value = serde_json::json!({"id": 1});

        while self
            .qmp
            .execute_with_args("guest-sync", arg.clone())
            .is_err()
        {
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    pub fn ping(&mut self) -> Result<(), Error> {
        self.qmp.execute("guest-ping").map(|_| ())
    }

    pub fn exec(&mut self, program: &str, args: &[String], env: &[String]) -> Result<u64, Error> {
        let arg = serde_json::json!({
            "path": program,
            "arg": args.to_vec(),
            "env": env.to_vec()
        });
        let response = self.qmp.execute_with_args("guest-exec", arg);
        if let Ok(QmpMessage::Success {
            ret: serde_json::Value::Object(fields),
            ..
        }) = response
        {
            if let Some(serde_json::Value::Number(pid)) = fields.get("pid") {
                return Ok(pid.as_u64().unwrap());
            }
        }

        Err(Error::ExecFailed)
    }

    pub fn get_exec_status(&mut self, pid: u64) -> Result<bool, Error> {
        let arg = serde_json::json!({"pid": pid});
        let response = self.qmp.execute_with_args("guest-exec-status", arg);
        if let Ok(QmpMessage::Success {
            ret: serde_json::Value::Object(fields),
            ..
        }) = response
        {
            if let Some(serde_json::Value::Bool(value)) = fields.get("exited") {
                return Ok(*value);
            }
        }

        Ok(false)
    }
}
