# Set environment variable
export CARGO_MAKE_EXTEND_WORKSPACE_MAKEFILE := "true"

# Run rustfmt to check the code formatting without making changes
format:
    cargo fmt -- --check

# Clean up the project by removing the target directory
clean:
    cargo clean

# Run clippy to catch common mistakes and improve your Rust code
clippy:
    RUSTFLAGS="-A unused" cargo clippy --all-targets --all-features -- -Dwarnings

# Generate documentation for the project
docs:
    cargo doc --no-deps

# Execute all unit tests in the workspace
test:
    cargo llvm-cov nextest --test-threads=1

# Run the entire CI pipeline including format, clippy, docs, and test checks
ci: format clippy docs test
    @echo "CI flow completed"
