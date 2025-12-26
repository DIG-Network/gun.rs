# Run all penetration test examples
# This script compiles and runs all example scripts sequentially

$ErrorActionPreference = "Continue"
$totalTests = 0
$passedTests = 0
$failedTests = 0

$categories = @(
    "01_gun_creation",
    "02_chain_basic",
    "03_chain_read",
    "04_chain_advanced",
    "05_chain_chaining",
    "06_sea_keys",
    "07_sea_signatures",
    "08_sea_encryption",
    "09_sea_users",
    "10_sea_certificates",
    "11_sea_utilities",
    "12_storage",
    "13_network",
    "14_errors",
    "15_edge_cases",
    "16_integration"
)

Write-Host "=== Running All Penetration Tests ===" -ForegroundColor Cyan
Write-Host ""

foreach ($category in $categories) {
    Write-Host "=== Category: $category ===" -ForegroundColor Yellow
    $categoryPath = "examples/penetration/$category"
    
    if (Test-Path $categoryPath) {
        $scripts = Get-ChildItem -Path $categoryPath -Filter "*.rs" | Sort-Object Name
        
        foreach ($script in $scripts) {
            $scriptName = $script.BaseName
            $scriptPath = $script.FullName
            $totalTests++
            
            Write-Host "  Testing: $scriptName..." -NoNewline
            
            # Compile and run
            $output = cargo run --example "penetration/$category/$scriptName" 2>&1
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
    } else {
        Write-Host "  Category directory not found: $categoryPath" -ForegroundColor Red
    }
    
    Write-Host ""
}

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

