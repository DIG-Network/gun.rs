# Script to run all penetration test examples and identify failures
$ErrorActionPreference = "Continue"
$results = @{
    Passed = @()
    Failed = @()
    Errors = @()
}

$examples = Get-ChildItem -Path $PSScriptRoot -Recurse -Filter *.rs | Where-Object { $_.Name -notmatch "run_|test_|compile_|convert_|fix_" }

Write-Host "Found $($examples.Count) examples to test" -ForegroundColor Cyan
Write-Host ""

$total = $examples.Count
$current = 0

foreach ($example in $examples) {
    $current++
    $relPath = $example.FullName.Replace("$PSScriptRoot\", "").Replace("\", "/")
    $exampleName = $example.BaseName
    
    Write-Host "[$current/$total] Testing: $relPath" -ForegroundColor Yellow -NoNewline
    
    # Copy to examples directory with a safe name
    $safeName = "penetration_test_" + [System.IO.Path]::GetFileNameWithoutExtension($example.Name).Replace(" ", "_").Replace("-", "_")
    $targetPath = Join-Path $PSScriptRoot ".." "$safeName.rs"
    Copy-Item -Path $example.FullName -Destination $targetPath -Force | Out-Null
    
    # Try to compile and run
    $compileOutput = & cargo check --example $safeName 2>&1
    $compileExitCode = $LASTEXITCODE
    
    if ($compileExitCode -ne 0) {
        Write-Host " - COMPILE FAILED" -ForegroundColor Red
        $results.Errors += @{
            Example = $relPath
            Error = "Compilation failed"
            Output = $compileOutput -join "`n"
        }
        Remove-Item -Path $targetPath -ErrorAction SilentlyContinue
        continue
    }
    
    # Run the example with timeout
    $runOutput = & timeout /t 30 /nobreak /nobreak 2>&1 | ForEach-Object { & cargo run --example $safeName 2>&1 }
    $runExitCode = $LASTEXITCODE
    
    # Clean up
    Remove-Item -Path $targetPath -ErrorAction SilentlyContinue
    
    if ($runExitCode -eq 0) {
        Write-Host " - PASSED" -ForegroundColor Green
        $results.Passed += $relPath
    } else {
        Write-Host " - FAILED (exit code: $runExitCode)" -ForegroundColor Red
        $results.Failed += @{
            Example = $relPath
            ExitCode = $runExitCode
            Output = $runOutput -join "`n"
        }
    }
}

Write-Host ""
Write-Host "=== SUMMARY ===" -ForegroundColor Cyan
Write-Host "Passed: $($results.Passed.Count)" -ForegroundColor Green
Write-Host "Failed: $($results.Failed.Count)" -ForegroundColor Red
Write-Host "Compile Errors: $($results.Errors.Count)" -ForegroundColor Red

# Save results
$resultsFile = Join-Path $PSScriptRoot "test_results.json"
$results | ConvertTo-Json -Depth 10 | Out-File -FilePath $resultsFile -Encoding UTF8
Write-Host ""
Write-Host "Results saved to: $resultsFile" -ForegroundColor Cyan

# Output failed examples
if ($results.Failed.Count -gt 0) {
    Write-Host ""
    Write-Host "Failed Examples:" -ForegroundColor Red
    $results.Failed | ForEach-Object { Write-Host "  - $($_.Example)" -ForegroundColor Yellow }
}

if ($results.Errors.Count -gt 0) {
    Write-Host ""
    Write-Host "Compilation Errors:" -ForegroundColor Red
    $results.Errors | ForEach-Object { Write-Host "  - $($_.Example)" -ForegroundColor Yellow }
}

