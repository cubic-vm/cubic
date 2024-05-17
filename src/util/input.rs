use std::io::{self, Write};

pub fn confirm(text: &str) -> bool {
    let mut reply = String::new();

    print!("{text}");
    std::io::stdout().flush().unwrap();
    io::stdin().read_line(&mut reply).unwrap();
    reply.trim() == "y"
}
