# Script to add global timeout wrapper to all examples
$examplesDir = Join-Path $PSScriptRoot ".."
$examples = Get-ChildItem -Path (Join-Path $examplesDir "penetration") -Recurse -Filter "*.rs" | 
    Where-Object { 
        $name = $_.Name
        $name -notmatch "run_|test_|compile_|convert_|fix_|README|FIXES|add_"
        (Get-Content $_.FullName -Raw) -match "#\[tokio::main\]"
    }

Write-Host "Found $($examples.Count) examples to add timeouts to" -ForegroundColor Cyan

$updated = 0
$skipped = 0

foreach ($file in $examples) {
    $content = Get-Content $file.FullName -Raw
    $originalContent = $content
    
    # Skip if already has timeout import and usage
    if ($content -match "use tokio::time::\{.*timeout" -and $content -match "timeout\(Duration::") {
        $skipped++
        continue
    }
    
    # Add timeout import if needed
    if ($content -match "use tokio::time::Duration;" -and $content -notmatch "use tokio::time::\{.*timeout") {
        $content = $content -replace 'use tokio::time::Duration;', 'use tokio::time::{Duration, timeout};'
    } elseif ($content -match "use tokio::time::Duration" -and $content -notmatch "use tokio::time::\{.*timeout") {
        # Handle case where it might be in a list
        $content = $content -replace 'use tokio::time::Duration([^}])', 'use tokio::time::{Duration, timeout}$1'
    }
    
    # Wrap the main body in a timeout - find the pattern: async fn main() { ... }
    # This is complex, so let's add timeout to .once() calls instead which is simpler and more targeted
    # For now, we'll just add the import and let manual fixes handle the wrapping
    
    if ($content -ne $originalContent) {
        Set-Content -Path $file.FullName -Value $content -NoNewline
        $updated++
        Write-Host "Updated: $($file.Name)" -ForegroundColor Green
    }
}

Write-Host "`nUpdated: $updated files" -ForegroundColor Green
Write-Host "Skipped (already has timeouts): $skipped files" -ForegroundColor Yellow

