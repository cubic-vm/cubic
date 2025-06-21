use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::util::Terminal;
use crate::view::SpinnerView;
use std::str;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const QEMU_GA_TIMEOUT_SECS: u64 = 300;

pub fn sh(instance_dao: &InstanceDao, verbosity: Verbosity, name: &str) -> Result<(), Error> {
    let instance = instance_dao.load(name)?;

    if !instance_dao.is_running(&instance) {
        commands::start(instance_dao, &None, verbosity, &vec![name.to_string()])?;
    }

    // Check if QEMU guest agent is present
    let ga_start = Instant::now();
    if let Ok(mut ga) = instance_dao.get_guest_agent(&instance) {
        let mut spinner = SpinnerView::new("Connecting to guest");
        while ga.ping().is_ok() || ga_start.elapsed() > Duration::from_secs(QEMU_GA_TIMEOUT_SECS) {
            thread::sleep(Duration::from_secs(1))
        }
        spinner.stop();
    }

    if ga_start.elapsed() > Duration::from_secs(QEMU_GA_TIMEOUT_SECS) {
        return Err(Error::MissingQemuGA);
    }

    let user = &instance.user;
    let sh = format!(
        "sh{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let chardev = &format!("{sh}_chardev");
    let device = &format!("{sh}_dev");

    {
        let mut monitor = instance_dao.get_monitor(&instance)?;
        monitor.add_unix_socket_chardev(chardev)?;
        monitor.add_virtserialport_device(device, chardev)?;
    }

    let console_path = format!("{}/{name}/{chardev}.socket", instance_dao.cache_dir);
    if let Ok(mut term) = Terminal::open(&console_path) {
        let pid;

        {
            let mut ga = instance_dao.get_guest_agent(&instance)?;
            ga.sync()?;

            pid = ga.exec(
                "sh",
                &[
                    "-c".to_string(),
                    format!("until [ -c /dev/virtio-ports/{device} ]; do sleep 1; done; socat /dev/virtio-ports/{device} exec:'su - {user}',raw,pty,stderr,setsid,sigint,sane,ctty"),
                ],
                &["TERM=linux".to_string()],
            )?;
        }

        while term.is_running() {
            {
                let mut ga = instance_dao.get_guest_agent(&instance)?;
                ga.sync()?;

                // update terminal geometry
                if let Some(termsize) = term.get_term_size() {
                    let (cols, rows) = termsize;
                    ga.exec(
                        "sh",
                        &[
                            "-c".to_string(),
                            format!(
                                "export CHILD=$(cat /proc/{pid}/task/{pid}/children | xargs);
                                export GRAND_CHILD=$(cat /proc/$CHILD/task/$CHILD/children | xargs);
                                stty -F /proc/$GRAND_CHILD/fd/0 cols {cols} rows {rows}"
                            ),
                        ],
                        &[],
                    )
                    .ok();
                }

                // check program status
                if let Ok(true) = ga.get_exec_status(pid) {
                    term.stop();
                }
            }

            thread::sleep(Duration::from_millis(100));
        }

        let mut ga = instance_dao.get_guest_agent(&instance)?;
        ga.sync()?;
        ga.exec("sh", &["-c".to_string(), format!("kill -9 {pid}")], &[])?;

        let mut monitor = instance_dao.get_monitor(&instance)?;
        monitor.delete_device(device)?;
        monitor.delete_chardev(chardev)?;
    }

    Ok(())
}
