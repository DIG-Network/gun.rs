# Simple script to run all examples
$cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
$examples = Get-ChildItem -Path examples -Filter "penetration_*.rs" | Sort-Object Name
$total = $examples.Count
$passed = 0
$failed = 0
$failedNames = @()

Write-Host "Running $total examples..." -ForegroundColor Cyan
Write-Host ""

for ($i = 0; $i -lt $total; $i++) {
    $name = $examples[$i].BaseName
    $num = $i + 1
    Write-Host "[$num/$total] $name..." -NoNewline
    
    $output = & $cargoPath run --example $name 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0) {
        Write-Host " OK" -ForegroundColor Green
        $passed++
    } else {
        Write-Host " FAILED" -ForegroundColor Red
        $failed++
        $failedNames += $name
    }
}

Write-Host ""
Write-Host "Summary: $passed passed, $failed failed out of $total" -ForegroundColor Cyan

if ($failed -gt 0) {
    Write-Host ""
    Write-Host "Failed examples:" -ForegroundColor Yellow
    $failedNames | ForEach-Object { Write-Host "  $_" -ForegroundColor Yellow }
}

exit $failed

