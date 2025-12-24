# Contributing to gun.rs

Thank you for your interest in contributing to gun.rs!

## Development Setup

1. Clone the repository
2. Install Rust (stable toolchain recommended)
3. Run `cargo build` to build the project
4. Run `cargo test` to run tests

## Code Quality Standards

### Formatting

All code must be formatted with `rustfmt`:

```bash
cargo fmt --all
```

### Linting

All code must pass `clippy` checks:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Testing

- Write unit tests for new functionality
- Add integration tests for network operations
- Ensure all tests pass before submitting PR

```bash
cargo test --all-features
```

### Lock Safety

When working with concurrent code:
- Never hold locks during async operations
- Minimize lock scope - acquire, clone data, release
- Avoid read-then-write lock upgrades
- Use timeouts for lock acquisition when appropriate

## Running Tests

### Unit Tests

```bash
cargo test --lib
```

### Integration Tests

```bash
cargo test --test integration_tests -- --test-threads=1
```

### Stress Tests

Stress tests are marked with `#[ignore]` and run only when explicitly requested:

```bash
cargo test --test stress_tests -- --ignored
```

### Lock Tests

Tests specifically for lock contention:

```bash
cargo test --test lock_tests
```

## Making Changes

1. Create a feature branch
2. Make your changes
3. Run `make ci` or `cargo fmt && cargo clippy && cargo test`
4. Commit with descriptive messages
5. Push and create a pull request

## Code Style

- Follow Rust conventions
- Use descriptive variable names
- Add comments for complex logic
- Document public APIs with rustdoc

## Reporting Issues

When reporting bugs, please include:
- Rust version
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs/output

