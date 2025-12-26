# Penetration Testing Examples

This directory contains comprehensive penetration testing examples for the Gun.rs library, covering all public API permutations.

## Structure

The examples are organized into 16 categories:

1. **01_gun_creation** - Gun instance creation (8 examples)
2. **02_chain_basic** - Chain API basic operations (8 examples)
3. **03_chain_read** - Chain API read operations (11 examples)
4. **04_chain_advanced** - Chain API advanced operations (10 examples)
5. **05_chain_chaining** - Chain API method chaining (11 examples)
6. **06_sea_keys** - SEA module key management (4 examples)
7. **07_sea_signatures** - SEA module signatures (6 examples)
8. **08_sea_encryption** - SEA module encryption (6 examples)
9. **09_sea_users** - SEA module user authentication (7 examples)
10. **10_sea_certificates** - SEA module certificates (6 examples)
11. **11_sea_utilities** - SEA module utilities (5 examples)
12. **12_storage** - Storage backends (6 examples)
13. **13_network** - Network operations (8 examples)
14. **14_errors** - Error handling (6 examples)
15. **15_edge_cases** - Edge cases and boundaries (8 examples)
16. **16_integration** - Integration scenarios (5 examples)

**Total: ~120 example scripts**

## Running Examples

### Option 1: Run Individual Examples

Since Rust's cargo doesn't automatically discover examples in subdirectories, you can run individual examples by:

1. Copy the example to the `examples/` directory with a unique name
2. Run with: `cargo run --example <name>`

For example:
```bash
cp examples/penetration/01_gun_creation/01_default.rs examples/penetration_01_default.rs
cargo run --example penetration_01_default
```

### Option 2: Use the Execution Scripts

The PowerShell scripts (`run_all.ps1` and `run_category.ps1`) are provided but require examples to be registered in `Cargo.toml` or copied to the `examples/` directory.

### Option 3: Manual Testing

You can manually test each example by:
1. Copying it to `examples/` with a unique name
2. Running `cargo run --example <name>`
3. Checking the exit code (0 = success, 1 = failure)

## Results Documentation

Results from test execution should be documented in:
- `results/working.md` - All working permutations
- `results/failing.md` - All failing permutations with error details
- `results/notes.md` - Observations, edge cases, performance notes

## Notes

- All examples are designed to be completely isolated (no shared state)
- Each example prints clear test name and description
- Examples exit with appropriate codes (0 = success, 1 = failure)
- Some examples may require network peers or storage setup to fully test

## Compilation

Most examples should compile successfully. If you encounter compilation errors:

1. Check that all required dependencies are in `Cargo.toml`
2. Verify imports match the actual API structure
3. Check for API changes that might affect example code

Common issues:
- Missing imports (e.g., `base64::Engine`)
- API signature changes
- Missing dependencies

