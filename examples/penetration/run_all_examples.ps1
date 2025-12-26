# Enhanced script to run all penetration test examples and categorize results
$cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
$ErrorActionPreference = "Continue"
$examplesDir = "examples"

Write-Host "=== Running All Penetration Test Examples ===" -ForegroundColor Cyan
Write-Host ""

# Get all example files
$exampleFiles = Get-ChildItem -Path $examplesDir -Filter "penetration_*.rs" | Sort-Object Name
$total = $exampleFiles.Count

$results = @()
$categories = @{
    Success = @()
    CompilationError = @()
    RuntimeError = @()
    LogicFailure = @()
    MissingOutput = @()
}

Write-Host "Running $total examples..." -ForegroundColor Cyan
Write-Host ""

for ($i = 0; $i -lt $total; $i++) {
    $file = $exampleFiles[$i]
    $exampleName = $file.BaseName
    $num = $i + 1
    
    Write-Host "[$num/$total] $exampleName..." -NoNewline
    
    try {
        # Run the example
        $output = & $cargoPath run --example $exampleName 2>&1 | Out-String
        $exitCode = $LASTEXITCODE
        
        # Analyze output
        $hasSuccess = $output -match "✓|Success"
        $hasFailure = $output -match "✗|Failed|FAILED|Error:|error:|Panic|panic"
        $hasSummary = $output -match "Summary"
        $compilationError = $output -match "error\[E|cannot|expected|found"
        
        # Categorize
        $category = "Unknown"
        $status = "UNKNOWN"
        
        if ($compilationError) {
            $category = "CompilationError"
            $status = "COMPILE_ERROR"
        } elseif ($exitCode -ne 0 -and $hasFailure) {
            if ($output -match "thread.*panicked|panicked at") {
                $category = "RuntimeError"
                $status = "RUNTIME_ERROR"
            } else {
                $category = "LogicFailure"
                $status = "LOGIC_FAILURE"
            }
        } elseif ($exitCode -eq 0 -and ($hasSuccess -or $hasSummary)) {
            $category = "Success"
            $status = "PASSED"
        } elseif ($exitCode -eq 0 -and -not $hasSuccess -and -not $hasFailure) {
            $category = "MissingOutput"
            $status = "NO_OUTPUT"
        } else {
            $category = "LogicFailure"
            $status = "FAILED"
        }
        
        # Display result
        switch ($category) {
            "Success" { Write-Host " PASSED" -ForegroundColor Green }
            "CompilationError" { Write-Host " COMPILE ERROR" -ForegroundColor Red }
            "RuntimeError" { Write-Host " RUNTIME ERROR" -ForegroundColor Red }
            "LogicFailure" { Write-Host " FAILED" -ForegroundColor Yellow }
            "MissingOutput" { Write-Host " NO OUTPUT" -ForegroundColor Magenta }
            default { Write-Host " UNKNOWN" -ForegroundColor Gray }
        }
        
        $categories[$category] += $exampleName
        
        $results += @{
            Name = $exampleName
            Status = $status
            Category = $category
            ExitCode = $exitCode
            HasSuccess = $hasSuccess
            HasFailure = $hasFailure
            HasSummary = $hasSummary
            Output = $output
        }
        
    } catch {
        Write-Host " ERROR" -ForegroundColor Red
        $categories["RuntimeError"] += $exampleName
        $results += @{
            Name = $exampleName
            Status = "ERROR"
            Category = "RuntimeError"
            ExitCode = -1
            Error = $_.ToString()
        }
    }
}

# Summary
Write-Host ""
Write-Host "=== Summary ===" -ForegroundColor Cyan
Write-Host "Total: $total"
Write-Host "Passed: $($categories.Success.Count)" -ForegroundColor Green
Write-Host "Compilation Errors: $($categories.CompilationError.Count)" -ForegroundColor Red
Write-Host "Runtime Errors: $($categories.RuntimeError.Count)" -ForegroundColor Red
Write-Host "Logic Failures: $($categories.LogicFailure.Count)" -ForegroundColor Yellow
Write-Host "Missing Output: $($categories.MissingOutput.Count)" -ForegroundColor Magenta

# Detailed breakdown
if ($categories.CompilationError.Count -gt 0) {
    Write-Host ""
    Write-Host "Compilation Errors:" -ForegroundColor Red
    $categories.CompilationError | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
}

if ($categories.RuntimeError.Count -gt 0) {
    Write-Host ""
    Write-Host "Runtime Errors:" -ForegroundColor Red
    $categories.RuntimeError | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
}

if ($categories.LogicFailure.Count -gt 0) {
    Write-Host ""
    Write-Host "Logic Failures:" -ForegroundColor Yellow
    $categories.LogicFailure | ForEach-Object { Write-Host "  $_" -ForegroundColor Yellow }
}

if ($categories.MissingOutput.Count -gt 0) {
    Write-Host ""
    Write-Host "Missing Output:" -ForegroundColor Magenta
    $categories.MissingOutput | ForEach-Object { Write-Host "  $_" -ForegroundColor Magenta }
}

# Save results
$timestamp = Get-Date -Format 'yyyyMMdd_HHmmss'
$resultsFile = "examples\penetration\run_results_$timestamp.json"
$results | ConvertTo-Json -Depth 3 | Out-File $resultsFile
Write-Host ""
Write-Host "Results saved to: $resultsFile" -ForegroundColor Gray

# Return exit code based on failures
$totalFailures = $categories.CompilationError.Count + $categories.RuntimeError.Count + $categories.LogicFailure.Count
exit $totalFailures
