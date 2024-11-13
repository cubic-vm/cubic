use crate::machine::{Machine, MachineDao, MachineState};
use crate::view::TimerView;
use std::thread::sleep;
use std::time::Duration;

pub struct InstanceStateChangeView {
    timer_view: TimerView,
    target_state: MachineState,
}

impl InstanceStateChangeView {
    pub fn new(message: &str, target_state: MachineState) -> Self {
        InstanceStateChangeView {
            timer_view: TimerView::new(message),
            target_state,
        }
    }

    pub fn run(&mut self, machine_dao: &MachineDao, machines: &[Machine]) {
        let mut done = false;
        while !done {
            done = true;
            for machine in machines {
                if machine_dao.get_state(machine) != self.target_state {
                    done = false;
                    break;
                }
            }
            self.timer_view.update();
            sleep(Duration::from_millis(10));
        }

        self.timer_view.done();
    }
}
