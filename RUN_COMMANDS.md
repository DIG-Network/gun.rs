# Quick Run Commands

## Run Two Clients Example

### With Full Path (Always Works)
```powershell
C:\Users\micha\.cargo\bin\cargo.exe run --example two_clients
```

### If Cargo is in PATH (After Restart)
```powershell
cargo run --example two_clients
```

## Other Useful Commands

### Build Only
```powershell
C:\Users\micha\.cargo\bin\cargo.exe build --example two_clients
```

### Run with Output
```powershell
C:\Users\micha\.cargo\bin\cargo.exe run --example two_clients 2>&1 | Select-Object -Last 100
```

### Run All Tests
```powershell
C:\Users\micha\.cargo\bin\cargo.exe test --all-features
```

### Format Code
```powershell
C:\Users\micha\.cargo\bin\cargo.exe fmt --all
```

### Clippy Check
```powershell
C:\Users\micha\.cargo\bin\cargo.exe clippy --all-targets --all-features -- -D warnings
```

