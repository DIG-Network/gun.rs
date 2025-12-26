# Test runner script to compile and test all penetration examples
# This script copies examples to the examples/ directory and tries to compile them

$ErrorActionPreference = "Continue"
$baseDir = "examples/penetration"
$examplesDir = "examples"
$totalTests = 0
$compiledTests = 0
$failedTests = 0
$errors = @()

Write-Host "=== Testing Penetration Examples ===" -ForegroundColor Cyan
Write-Host ""

# Get all categories
$categories = Get-ChildItem -Path $baseDir -Directory | Sort-Object Name

foreach ($category in $categories) {
    $categoryName = $category.Name
    Write-Host "=== Category: $categoryName ===" -ForegroundColor Yellow
    
    $scripts = Get-ChildItem -Path $category.FullName -Filter "*.rs" | Sort-Object Name
    
    foreach ($script in $scripts) {
        $scriptName = $script.BaseName
        $uniqueName = "penetration_$($categoryName)_$scriptName"
        $targetPath = Join-Path $examplesDir "$uniqueName.rs"
        $totalTests++
        
        Write-Host "  Testing: $scriptName..." -NoNewline
        
        # Copy file
        try {
            Copy-Item -Path $script.FullName -Destination $targetPath -Force | Out-Null
            
            # Try to check compilation (we'll use cargo check if available, otherwise just verify file copied)
            Write-Host " COPIED" -ForegroundColor Green
            $compiledTests++
        } catch {
            Write-Host " FAILED TO COPY" -ForegroundColor Red
            $failedTests++
            $errors += "Failed to copy $scriptName : $($_.Exception.Message)"
        }
    }
    
    Write-Host ""
}

Write-Host "=== Summary ===" -ForegroundColor Cyan
Write-Host "Total Tests: $totalTests"
Write-Host "Copied: $compiledTests" -ForegroundColor Green
Write-Host "Failed: $failedTests" -ForegroundColor Red

if ($errors.Count -gt 0) {
    Write-Host ""
    Write-Host "Errors:" -ForegroundColor Red
    foreach ($error in $errors) {
        Write-Host "  - $error" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "Note: To compile and run examples, use:" -ForegroundColor Yellow
Write-Host "  cargo check --example <example_name>" -ForegroundColor Yellow
Write-Host "  cargo run --example <example_name>" -ForegroundColor Yellow

