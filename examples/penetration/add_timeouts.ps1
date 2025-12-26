# Script to add timeouts to all penetration test examples
$examplesDir = Join-Path $PSScriptRoot ".."
$examples = Get-ChildItem -Path (Join-Path $examplesDir "penetration") -Recurse -Filter "*.rs" | 
    Where-Object { $_.Name -notmatch "run_|test_|compile_|convert_|fix_|README|FIXES" }

Write-Host "Found $($examples.Count) examples to update with timeouts" -ForegroundColor Cyan

$timeoutPattern = 'use tokio::time::Duration;'
$timeoutImport = "use tokio::time::{Duration, timeout};"

foreach ($file in $examples) {
    $content = Get-Content $file.FullName -Raw
    $modified = $false
    
    # Add timeout import if not present
    if ($content -match "use tokio::time::Duration;" -and $content -notmatch "use tokio::time::\{.*timeout") {
        $content = $content -replace 'use tokio::time::Duration;', $timeoutImport
        $modified = $true
    }
    
    if ($modified) {
        Set-Content -Path $file.FullName -Value $content -NoNewline
        Write-Host "Updated: $($file.Name)" -ForegroundColor Green
    }
}

Write-Host "`nDone adding timeout imports" -ForegroundColor Cyan

