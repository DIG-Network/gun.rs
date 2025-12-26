# Run penetration test examples and verify outputs
param(
    [int]$StartIndex = 0,
    [int]$Count = 10
)

$cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
$ErrorActionPreference = "Continue"
$examplesDir = "examples"

# Get all example files
$exampleFiles = Get-ChildItem -Path $examplesDir -Filter "penetration_*.rs" | Sort-Object Name
$total = $exampleFiles.Count
$endIndex = if ($Count -gt 0) { [Math]::Min($StartIndex + $Count - 1, $total - 1) } else { $total - 1 }

Write-Host "=== Running Examples $($StartIndex + 1) to $($endIndex + 1) of $total ===" -ForegroundColor Cyan

$results = @()
for ($i = $StartIndex; $i -le $endIndex; $i++) {
    $file = $exampleFiles[$i]
    $exampleName = $file.BaseName
    $num = $i + 1
    
    Write-Host "[$num/$total] $exampleName..." -NoNewline
    
    try {
        $output = & $cargoPath run --example $exampleName 2>&1 | Out-String
        $exitCode = $LASTEXITCODE
        
        # Check output for success/failure indicators
        $hasSuccess = $output -match "✓|Success|success|PASSED"
        $hasFailure = $output -match "✗|Failed|failed|FAILED|Error:|error:|Panic|panic"
        $hasSummary = $output -match "Summary"
        
        if ($exitCode -eq 0) {
            if ($hasFailure) {
                Write-Host " FAILED (has failure markers)" -ForegroundColor Red
                $status = "FAILED"
            } elseif ($hasSuccess -or $hasSummary) {
                Write-Host " PASSED" -ForegroundColor Green
                $status = "PASSED"
            } else {
                Write-Host " UNKNOWN (no clear success/failure)" -ForegroundColor Yellow
                $status = "UNKNOWN"
            }
        } else {
            Write-Host " FAILED (exit code: $exitCode)" -ForegroundColor Red
            $status = "FAILED"
        }
        
        $results += @{
            Name = $exampleName
            Status = $status
            ExitCode = $exitCode
            HasSuccess = $hasSuccess
            HasFailure = $hasFailure
            HasSummary = $hasSummary
        }
    } catch {
        Write-Host " ERROR: $_" -ForegroundColor Red
        $results += @{
            Name = $exampleName
            Status = "ERROR"
            ExitCode = -1
            Error = $_.ToString()
        }
    }
}

Write-Host ""
$passed = ($results | Where-Object { $_.Status -eq "PASSED" }).Count
$failed = ($results | Where-Object { $_.Status -eq "FAILED" }).Count
$unknown = ($results | Where-Object { $_.Status -eq "UNKNOWN" }).Count

Write-Host "Results: $passed passed, $failed failed, $unknown unknown" -ForegroundColor Cyan

# Save results
$resultsFile = "examples\penetration\run_results_$StartIndex.json"
$results | ConvertTo-Json -Depth 3 | Out-File $resultsFile
Write-Host "Results saved to: $resultsFile" -ForegroundColor Gray

return $results

