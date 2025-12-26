# Script to convert all penetration test examples to two-client approach
# This performs automated conversions that can be verified and adjusted

$relayUrl = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun"

Write-Host "Converting penetration test examples to two-client approach..." -ForegroundColor Cyan

# Get all example files (both in subdirectories and root)
$files = Get-ChildItem -Path "examples" -Filter "penetration*.rs" -Recurse | Where-Object {
    $_.FullName -notmatch "convert.*\.ps1"
}

$converted = 0
$skipped = 0

foreach ($file in $files) {
    $content = Get-Content $file.FullName -Raw -Encoding UTF8
    $originalContent = $content
    
    # Skip if already converted (has GunOptions and client1/client2 pattern)
    if ($content -match "GunOptions" -and ($content -match "client1|Client 1")) {
        Write-Host "SKIP: $($file.Name) (already converted)" -ForegroundColor Gray
        $skipped++
        continue
    }
    
    $modified = $false
    
    # 1. Update imports
    if ($content -match "use gun::Gun;") {
        $content = $content -replace "use gun::Gun;", "use gun::{Gun, GunOptions};"
        $modified = $true
    }
    
    # 2. Add relay URL constant (if not present)
    if (-not ($content -match "const RELAY_URL")) {
        # Add after the last use statement
        if ($content -match "(use .+;\s*\n)") {
            $content = $content -replace "(use .+;\s*\n)", "`$1`nconst RELAY_URL: &str = `"$relayUrl`";`n"
            $modified = $true
        }
    }
    
    # 3. Basic pattern: Replace "let gun = Gun::new();" with two clients
    # This is a simple case - more complex ones need manual conversion
    if ($content -match "let gun = Gun::new\(\);") {
        Write-Host "NOTE: $($file.Name) needs manual conversion (has Gun::new())" -ForegroundColor Yellow
        # Don't auto-convert this - needs context-specific handling
    }
    
    if ($modified -and $content -ne $originalContent) {
        try {
            Set-Content -Path $file.FullName -Value $content -Encoding UTF8 -NoNewline
            Write-Host "CONVERTED: $($file.Name)" -ForegroundColor Green
            $converted++
        } catch {
            Write-Host "ERROR converting $($file.Name): $_" -ForegroundColor Red
        }
    } else {
        Write-Host "SKIP: $($file.Name) (no simple conversions needed)" -ForegroundColor Gray
        $skipped++
    }
}

Write-Host "`nConversion Summary:" -ForegroundColor Cyan
Write-Host "  Converted: $converted" -ForegroundColor Green
Write-Host "  Skipped: $skipped" -ForegroundColor Yellow
Write-Host "`nNote: Most examples will need manual conversion of Gun::new() to two-client pattern" -ForegroundColor Yellow

