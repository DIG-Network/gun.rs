# Compile and fix all penetration test examples
# This script tries to compile each example and reports errors

$ErrorActionPreference = "Continue"
$baseDir = "examples/penetration"
$examplesDir = "examples"
$totalTests = 0
$compiledTests = 0
$failedTests = 0
$errors = @()

Write-Host "=== Compiling Penetration Examples ===" -ForegroundColor Cyan
Write-Host ""

# Get all categories
$categories = Get-ChildItem -Path $baseDir -Directory | Sort-Object Name | Where-Object { $_.Name -ne "results" }

foreach ($category in $categories) {
    $categoryName = $category.Name
    Write-Host "=== Category: $categoryName ===" -ForegroundColor Yellow
    
    $scripts = Get-ChildItem -Path $category.FullName -Filter "*.rs" | Sort-Object Name
    
    foreach ($script in $scripts) {
        $scriptName = $script.BaseName
        $uniqueName = "penetration_$($categoryName)_$scriptName"
        $targetPath = Join-Path $examplesDir "$uniqueName.rs"
        $totalTests++
        
        Write-Host "  Checking: $scriptName..." -NoNewline
        
        # Try to check compilation
        $checkOutput = & cargo check --example $uniqueName 2>&1
        $exitCode = $LASTEXITCODE
        
        if ($exitCode -eq 0) {
            Write-Host " OK" -ForegroundColor Green
            $compiledTests++
        } else {
            Write-Host " FAILED" -ForegroundColor Red
            $failedTests++
            $errorMsg = ($checkOutput | Out-String).Trim()
            $errors += @{
                File = $scriptName
                Category = $categoryName
                Error = $errorMsg
            }
            Write-Host "    Error: $($errorMsg -split "`n" | Select-Object -First 3 -join "`n    ")" -ForegroundColor Red
        }
    }
    
    Write-Host ""
}

Write-Host "=== Summary ===" -ForegroundColor Cyan
Write-Host "Total Tests: $totalTests"
Write-Host "Compiled: $compiledTests" -ForegroundColor Green
Write-Host "Failed: $failedTests" -ForegroundColor Red

if ($errors.Count -gt 0) {
    Write-Host ""
    Write-Host "=== Errors ===" -ForegroundColor Red
    foreach ($error in $errors) {
        Write-Host "  $($error.Category)/$($error.File):" -ForegroundColor Yellow
        Write-Host "    $($error.Error -split "`n" | Select-Object -First 5 -join "`n    ")" -ForegroundColor Red
    }
}

exit $failedTests

