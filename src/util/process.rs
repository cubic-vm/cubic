use std::process::Command;

pub fn print_command(command: &Command) {
    print!("{}", command.get_program().to_str().unwrap_or("n/a"));
    for arg in command.get_args() {
        print!(" {}", arg.to_str().unwrap_or("n/a"));
    }
    println!();
}
