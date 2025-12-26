# Compile all penetration test examples and report errors
$cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
$ErrorActionPreference = "Continue"
$examplesDir = "examples"
$totalTests = 0
$compiledTests = 0
$failedTests = 0
$errors = @()

Write-Host "=== Compiling All Penetration Examples ===" -ForegroundColor Cyan
Write-Host "Using cargo: $cargoPath" -ForegroundColor Gray
Write-Host ""

# Get all example files
$exampleFiles = Get-ChildItem -Path $examplesDir -Filter "penetration_*.rs" | Sort-Object Name

foreach ($file in $exampleFiles) {
    $exampleName = $file.BaseName
    $totalTests++
    
    Write-Host "[$totalTests/$($exampleFiles.Count)] Checking: $exampleName..." -NoNewline
    
    # Try to check compilation
    $checkOutput = & $cargoPath check --example $exampleName 2>&1
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0) {
        Write-Host " ✓ OK" -ForegroundColor Green
        $compiledTests++
    } else {
        Write-Host " ✗ FAILED" -ForegroundColor Red
        $failedTests++
        $errorMsg = ($checkOutput | Out-String).Trim()
        $errors += @{
            File = $exampleName
            Error = $errorMsg
        }
        # Show first few lines of error
        $errorLines = $errorMsg -split "`n" | Select-Object -First 5
        foreach ($line in $errorLines) {
            if ($line.Trim() -ne "") {
                Write-Host "    $line" -ForegroundColor Red
            }
        }
    }
}

Write-Host ""
Write-Host "=== Summary ===" -ForegroundColor Cyan
Write-Host "Total: $totalTests"
Write-Host "Compiled: $compiledTests" -ForegroundColor Green
Write-Host "Failed: $failedTests" -ForegroundColor Red

if ($errors.Count -gt 0) {
    Write-Host ""
    Write-Host "=== Failed Examples ===" -ForegroundColor Yellow
    foreach ($error in $errors) {
        Write-Host "  $($error.File)" -ForegroundColor Yellow
    }
}

# Save errors to file
if ($errors.Count -gt 0) {
    $errors | ConvertTo-Json -Depth 3 | Out-File "examples\penetration\compilation_errors.json"
    Write-Host ""
    Write-Host "Errors saved to: examples\penetration\compilation_errors.json" -ForegroundColor Gray
}

exit $failedTests

