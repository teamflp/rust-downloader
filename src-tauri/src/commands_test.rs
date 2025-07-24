
#[cfg(test)]
mod tests {
    use crate::commands::check_command;

    #[test]
    fn test_check_command_exists() {
        // Test with a command that should exist on most systems
        assert!(check_command("ls"));
    }

    #[test]
    fn test_check_command_not_exists() {
        // Test with a command that should not exist
        assert!(!check_command("this_command_does_not_exist_12345"));
    }
}