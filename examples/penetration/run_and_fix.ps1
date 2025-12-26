# Script to run examples and identify failures
$cargoPath = "C:\Users\micha\workspace\gun.rs\.cargo\bin\cargo.exe"
if (-not (Test-Path $cargoPath)) {
    $cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
}

$ErrorActionPreference = "Continue"
$examplesDir = Join-Path $PSScriptRoot ".."

Set-Location (Split-Path $PSScriptRoot -Parent)

# Get all penetration examples
$examples = Get-ChildItem -Path $examplesDir -Filter "penetration_*.rs" | 
    Sort-Object Name |
    Select-Object -First $args[0]

if ($null -eq $examples -or $examples.Count -eq 0) {
    Write-Host "No examples found or invalid count argument" -ForegroundColor Red
    exit 1
}

Write-Host "Running $($examples.Count) examples..." -ForegroundColor Cyan
Write-Host ""

$results = @{
    Passed = @()
    Failed = @()
    CompileErrors = @()
}

foreach ($ex in $examples) {
    $name = $ex.BaseName
    Write-Host "[$($results.Passed.Count + $results.Failed.Count + $results.CompileErrors.Count + 1)/$($examples.Count)] $name..." -NoNewline -ForegroundColor Yellow
    
    # Check compilation first
    $compileOutput = & $cargoPath check --example $name 2>&1 | Out-String
    $compileExitCode = $LASTEXITCODE
    
    if ($compileExitCode -ne 0) {
        Write-Host " COMPILE ERROR" -ForegroundColor Red
        $results.CompileErrors += @{
            Name = $name
            Error = $compileOutput
        }
        continue
    }
    
    # Run the example
    try {
        $runOutput = & $cargoPath run --example $name 2>&1 | Out-String
        $runExitCode = $LASTEXITCODE
        
        # Check for success indicators
        $hasSuccessMsg = $runOutput -match "Success.*:\s*[1-9]|✓.*Success|Summary.*Success"
        $hasFailureMsg = $runOutput -match "Failed.*:\s*[1-9]|✗.*Failed|FAILED"
        $exitedWithZero = $runExitCode -eq 0
        
        if ($exitedWithZero -and ($hasSuccessMsg -or -not $hasFailureMsg)) {
            Write-Host " PASSED" -ForegroundColor Green
            $results.Passed += $name
        } else {
            Write-Host " FAILED (exit: $runExitCode)" -ForegroundColor Red
            $results.Failed += @{
                Name = $name
                ExitCode = $runExitCode
                Output = $runOutput
            }
        }
    } catch {
        Write-Host " ERROR" -ForegroundColor Red
        $results.Failed += @{
            Name = $name
            Error = $_.ToString()
        }
    }
}

Write-Host ""
Write-Host "=== Results ===" -ForegroundColor Cyan
Write-Host "Passed: $($results.Passed.Count)" -ForegroundColor Green
Write-Host "Failed: $($results.Failed.Count)" -ForegroundColor Red
Write-Host "Compile Errors: $($results.CompileErrors.Count)" -ForegroundColor Red

if ($results.Failed.Count -gt 0) {
    Write-Host ""
    Write-Host "Failed examples:" -ForegroundColor Red
    $results.Failed | ForEach-Object { Write-Host "  - $($_.Name)" -ForegroundColor Yellow }
}

if ($results.CompileErrors.Count -gt 0) {
    Write-Host ""
    Write-Host "Compile errors:" -ForegroundColor Red
    $results.CompileErrors | ForEach-Object { Write-Host "  - $($_.Name)" -ForegroundColor Yellow }
}

return $results

