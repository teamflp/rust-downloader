# Development Guidelines for Rust Media Downloader

This document provides essential information for developers working on the Rust Media Downloader project.

## Build/Configuration Instructions

### Prerequisites
- Rust and Cargo (latest stable version recommended)
- External dependencies:
  - yt-dlp: Used for downloading media
  - ffmpeg: Used for media processing
  - curl: Used for network operations

### Building the Project
1. Clone the repository:
   ```bash
   git clone https://github.com/teamflp/rust-downloader.git
   cd rust-downloader
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run the application:
   ```bash
   cargo run
   ```

4. Build for release:
   ```bash
   cargo build --release
   ```

### Cross-Platform Compilation
The project supports multiple platforms. To build for a specific platform:

```bash
# For Windows (requires the appropriate target)
cargo build --target x86_64-pc-windows-gnu --release

# For macOS
cargo build --target x86_64-apple-darwin --release
```

## Testing Information

### Running Tests
The project uses Rust's built-in testing framework. To run all tests:

```bash
cargo test
```

To run tests with output:

```bash
cargo test -- --nocapture
```

You can also use the alias defined in Cargo.toml:

```bash
cargo tests
```

### Adding New Tests
1. Tests can be added in two ways:
   - In the same file as the code being tested, within a `#[cfg(test)]` module
   - In separate files with names ending in `_test.rs` or `_tests.rs`

2. Test structure:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;  // Import the module being tested

       #[test]
       fn test_function_name() {
           // Test code here
           assert!(true);  // Example assertion
       }
   }
   ```

3. Example test:
   ```rust
   #[test]
   fn test_check_command_exists() {
       // Test with a command that should exist on most systems
       assert!(check_command("ls"));
   }
   ```

### Test Example
Here's a simple test that verifies the `check_command` function works correctly:

```rust
// In src/commands_test.rs
use super::commands::check_command;

#[cfg(test)]
mod tests {
    use super::*;

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
```

## Additional Development Information

### Code Style
- Follow the Rust standard style guide
- Use `cargo fmt` to format code (alias: `cargo fmt`)
- Use meaningful variable and function names
- Add comments for complex logic
- Use French for user-facing messages (as seen in the codebase)

### Project Structure
- `src/main.rs`: Entry point of the application
- `src/commands.rs`: Command handling utilities
- `src/downloader.rs`: Core functionality for downloading media
- `src/installers.rs`: Dependency installation handling
- `src/progress.rs`: Progress bar and download progress tracking
- `src/user_input.rs`: User input handling

### Error Handling
- Use `Result` and `Option` types for error handling
- Provide meaningful error messages
- Handle edge cases appropriately

### Dependencies
The project uses several external dependencies:
- `indicatif`: For progress bars
- `dirs`: For directory handling
- `regex`: For regular expressions
- `colored`: For colored terminal output
- `which`: For finding executables in PATH

### Release Process
The project uses GitHub Actions for CI/CD. The release process is automated through the `.github/workflows/release.yml` workflow.

To create a new release:
1. Update the version in `Cargo.toml`
2. Commit and push the changes
3. Create a new tag with the version number
4. Push the tag to trigger the release workflow