# Simple batch test script
$cargoPath = "C:\Users\micha\.cargo\bin\cargo.exe"
$ErrorActionPreference = "Continue"

$examples = Get-ChildItem -Path (Join-Path $PSScriptRoot "..") -Filter "penetration_*.rs" | 
    Sort-Object Name | 
    Select-Object -First $args[0]

if ($null -eq $examples -or $examples.Count -eq 0) {
    Write-Host "No examples found or invalid count argument" -ForegroundColor Red
    exit 1
}

Write-Host "Testing $($examples.Count) examples..." -ForegroundColor Cyan
Write-Host ""

$results = @{
    Passed = @()
    Failed = @()
}

Set-Location (Split-Path $PSScriptRoot -Parent)

foreach ($ex in $examples) {
    $name = $ex.BaseName
    Write-Host "$name..." -NoNewline -ForegroundColor Yellow
    
    $output = & $cargoPath run --example $name 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    # Check for success indicators in output
    $hasSuccessMsg = $output -match "Success.*:.*[1-9]|✓.*Success"
    $hasFailureMsg = $output -match "Failed.*:.*[1-9]|✗.*Failed"
    $exitedWithZero = $exitCode -eq 0
    
    # Consider it passed if exit code is 0 AND has success message OR no failure message
    if ($exitedWithZero -and ($hasSuccessMsg -or -not $hasFailureMsg)) {
        Write-Host " PASSED" -ForegroundColor Green
        $results.Passed += $name
    } else {
        Write-Host " FAILED (exit: $exitCode)" -ForegroundColor Red
        $results.Failed += @{
            Name = $name
            ExitCode = $exitCode
            Output = $output
        }
    }
}

Write-Host ""
Write-Host "=== Results ===" -ForegroundColor Cyan
Write-Host "Passed: $($results.Passed.Count)" -ForegroundColor Green
Write-Host "Failed: $($results.Failed.Count)" -ForegroundColor Red

if ($results.Failed.Count -gt 0) {
    Write-Host ""
    Write-Host "Failed examples:" -ForegroundColor Red
    $results.Failed | ForEach-Object { Write-Host "  - $($_.Name)" -ForegroundColor Yellow }
}

return $results

