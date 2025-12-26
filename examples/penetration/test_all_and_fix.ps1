# Script to test all penetration examples and fix failures
$cargoPath = "C:\Users\micha\.cargo\bin\cargo.exe"
$ErrorActionPreference = "Continue"
$examplesDir = Join-Path $PSScriptRoot ".."
$workingDir = Split-Path $PSScriptRoot -Parent

Set-Location $workingDir

Write-Host "=== Testing All Penetration Examples ===" -ForegroundColor Cyan
Write-Host ""

# Get all example files from penetration directory
$exampleFiles = Get-ChildItem -Path (Join-Path $examplesDir "penetration") -Recurse -Filter "*.rs" | 
    Where-Object { $_.Name -notmatch "run_|test_|compile_|convert_|fix_|README|FIXES" } |
    Sort-Object FullName

# Also get examples from root examples directory that match penetration pattern
$rootExamples = Get-ChildItem -Path $examplesDir -Filter "penetration_*.rs" | Sort-Object Name

$allExamples = $exampleFiles + $rootExamples
$total = $allExamples.Count

Write-Host "Found $total examples to test" -ForegroundColor Cyan
Write-Host ""

$results = @{
    Passed = @()
    Failed = @()
    CompileErrors = @()
}

for ($i = 0; $i -lt $total; $i++) {
    $file = $allExamples[$i]
    $exampleName = $file.BaseName
    $num = $i + 1
    
    Write-Host "[$num/$total] $exampleName..." -NoNewline -ForegroundColor Yellow
    
    try {
        # Try to compile first
        $compileOutput = & $cargoPath check --example $exampleName 2>&1 | Out-String
        $compileExitCode = $LASTEXITCODE
        
        if ($compileExitCode -ne 0) {
            Write-Host " COMPILE ERROR" -ForegroundColor Red
            $results.CompileErrors += @{
                Name = $exampleName
                Path = $file.FullName
                Error = $compileOutput
            }
            continue
        }
        
        # Run the example with timeout
        $runOutput = & $cargoPath run --example $exampleName 2>&1 | Out-String
        $runExitCode = $LASTEXITCODE
        
        # Check output for success indicators
        $hasSuccess = $runOutput -match "✓|Success.*:.*\d+|PASSED|Summary.*Success"
        $hasFailure = $runOutput -match "✗|Failed.*:.*\d+|FAILED"
        $exitedNonZero = $runExitCode -ne 0
        
        if ($exitedNonZero -or ($hasFailure -and -not $hasSuccess)) {
            Write-Host " FAILED" -ForegroundColor Red
            $results.Failed += @{
                Name = $exampleName
                Path = $file.FullName
                ExitCode = $runExitCode
                Output = $runOutput
            }
        } else {
            Write-Host " PASSED" -ForegroundColor Green
            $results.Passed += $exampleName
        }
        
    } catch {
        Write-Host " ERROR" -ForegroundColor Red
        $results.Failed += @{
            Name = $exampleName
            Path = $file.FullName
            Error = $_.ToString()
        }
    }
}

Write-Host ""
Write-Host "=== SUMMARY ===" -ForegroundColor Cyan
Write-Host "Passed: $($results.Passed.Count)" -ForegroundColor Green
Write-Host "Failed: $($results.Failed.Count)" -ForegroundColor Red
Write-Host "Compile Errors: $($results.CompileErrors.Count)" -ForegroundColor Red

# Save results
$resultsFile = Join-Path $PSScriptRoot "test_results_$(Get-Date -Format 'yyyyMMdd_HHmmss').json"
$results | ConvertTo-Json -Depth 10 | Out-File -FilePath $resultsFile -Encoding UTF8
Write-Host ""
Write-Host "Results saved to: $resultsFile" -ForegroundColor Cyan

# Return results for further processing
return $results

