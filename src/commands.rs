use std::process::Command;

pub fn check_command(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", cmd))
        .output()
        .map_or(false, |output| output.status.success())
}
