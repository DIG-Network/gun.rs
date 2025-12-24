# Adding Cargo to PATH on Windows

If you need to add Cargo to your PATH permanently, here are the methods:

## Method 1: PowerShell (Recommended - Already Done)

The following command has been run to add Cargo to your PATH:

```powershell
$cargoPath = "$env:USERPROFILE\.cargo\bin"
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$cargoPath*") {
    $newPath = $currentPath + ";$cargoPath"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "Added Cargo to PATH"
}
```

## Method 2: Manual via Windows Settings

1. Press `Win + R`, type `sysdm.cpl`, press Enter
2. Click "Environment Variables"
3. Under "User variables", select "Path" and click "Edit"
4. Click "New" and add: `%USERPROFILE%\.cargo\bin`
5. Click "OK" on all windows

## Method 3: Command Prompt (as Administrator)

```cmd
setx PATH "%PATH%;%USERPROFILE%\.cargo\bin"
```

## Verifying

To verify Cargo is in your PATH:

**PowerShell:**
```powershell
cargo --version
```

**Command Prompt:**
```cmd
cargo --version
```

**Note:** You may need to:
- Close and reopen your terminal
- Log out and log back in
- Restart your computer (for system-wide changes)

## Current Session

To use Cargo in your current terminal session without restarting:

**PowerShell:**
```powershell
$env:Path += ";$env:USERPROFILE\.cargo\bin"
```

**Command Prompt:**
```cmd
set PATH=%PATH%;%USERPROFILE%\.cargo\bin
```

## Troubleshooting

If `cargo --version` doesn't work:
1. Verify the path exists: `Test-Path "$env:USERPROFILE\.cargo\bin\cargo.exe"`
2. Check if it's in PATH: `$env:Path -split ';' | Select-String cargo`
3. Restart your terminal or computer

