# Run specific category of penetration test examples
# Usage: .\run_category.ps1 <category_name>
# Example: .\run_category.ps1 01_gun_creation

param(
    [Parameter(Mandatory=$true)]
    [string]$Category
)

$ErrorActionPreference = "Continue"
$totalTests = 0
$passedTests = 0
$failedTests = 0

$categoryPath = "examples/penetration/$Category"

if (-not (Test-Path $categoryPath)) {
    Write-Host "Category directory not found: $categoryPath" -ForegroundColor Red
    exit 1
}

Write-Host "=== Running Category: $Category ===" -ForegroundColor Cyan
Write-Host ""

$scripts = Get-ChildItem -Path $categoryPath -Filter "*.rs" | Sort-Object Name

foreach ($script in $scripts) {
    $scriptName = $script.BaseName
    $scriptPath = $script.FullName
    $totalTests++
    
    Write-Host "  Testing: $scriptName..." -NoNewline
    
    # Compile and run
    $output = cargo run --example "penetration/$Category/$scriptName" 2>&1
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0) {
        Write-Host " PASSED" -ForegroundColor Green
        $passedTests++
    } else {
        Write-Host " FAILED" -ForegroundColor Red
        $failedTests++
        Write-Host "    Error output:" -ForegroundColor Red
        Write-Host $output -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "=== Summary ===" -ForegroundColor Cyan
Write-Host "Total Tests: $totalTests"
Write-Host "Passed: $passedTests" -ForegroundColor Green
Write-Host "Failed: $failedTests" -ForegroundColor Red

if ($failedTests -eq 0) {
    Write-Host "All tests passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "Some tests failed." -ForegroundColor Red
    exit 1
}

