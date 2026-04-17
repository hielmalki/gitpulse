# Contributing to GitPulse

Thank you for your interest in contributing!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/gitpulse`
3. Install Rust: https://rustup.rs
4. Build: `cargo build`
5. Run tests: `cargo test`

## Development

```bash
# Run with a test username
GITHUB_TOKEN=your_token cargo run -- dashboard torvalds

# Check for issues
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt
```

## Pull Request Guidelines

- One feature/fix per PR
- Conventional Commit messages (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`)
- Run `cargo clippy` and `cargo fmt` before submitting
- Add tests for new scoring rules or API calls

## Reporting Bugs

Open an issue with:
- Your OS and Rust version (`rustc --version`)
- The `gitpulse` version (`gitpulse --version`)
- Steps to reproduce
- Expected vs. actual behavior
