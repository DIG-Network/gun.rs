# Instructions for Updating Remaining Files

Due to the large number of files (181+ example files and many test files), here's a guide for updating the remaining files:

## Pattern 1: Gun::new()

Replace:
```rust
let gun = Gun::new();
```

With:
```rust
let secret_key = SecretKey::from_seed(&[0u8; 32]);
let public_key = secret_key.public_key();
let gun = Gun::new(secret_key, public_key);
```

## Pattern 2: Arc::new(Gun::new())

Replace:
```rust
let gun = Arc::new(Gun::new());
```

With:
```rust
let secret_key = SecretKey::from_seed(&[0u8; 32]);
let public_key = secret_key.public_key();
let gun = Arc::new(Gun::new(secret_key, public_key));
```

## Pattern 3: Gun::with_options(options)

Replace:
```rust
let options = GunOptions { ... };
let gun = Gun::with_options(options).await?;
```

With:
```rust
let secret_key = SecretKey::from_seed(&[0u8; 32]);
let public_key = secret_key.public_key();
let options = GunOptions { ... };
let gun = Gun::with_options(secret_key, public_key, options).await?;
```

## Pattern 4: Multiple instances in same file

For files with multiple Gun instances, use different seed values:
- First instance: `&[0u8; 32]` or `&[1u8; 32]`
- Second instance: `&[2u8; 32]`
- Third instance: `&[3u8; 32]`
- etc.

## Required Import

Add at the top of each file (if not already present):
```rust
use chia_bls::{SecretKey, PublicKey};
```

## Files Already Updated

- `src/lib.rs` (test)
- `src/bin/server.rs`
- `examples/penetration_14_errors_06_error_propagation.rs`
- `examples/penetration_04_chain_advanced_07_back_single.rs`
- `examples/penetration_01_gun_creation_03_with_options_storage.rs`
- `examples/penetration_16_integration_02_multi_user_chat.rs`
- `examples/penetration/03_chain_read/01_once_basic.rs`
- `examples/penetration_01_gun_creation_04_with_options_peers.rs`
- `tests/basic_tests.rs`
- `tests/gun_instance_tests.rs`
- `tests/integration_tests.rs` (partially)

## Remaining Files

Run this command to find all remaining files:
```bash
grep -r "Gun::new()" examples/ tests/ --include="*.rs"
grep -r "Gun::with_options(" examples/ tests/ --include="*.rs"
```

