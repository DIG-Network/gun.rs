# Script to help convert examples to two-client approach
# This identifies patterns but conversion needs manual verification

$relayUrl = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun"

Write-Host "Conversion helper for two-client approach" -ForegroundColor Cyan
Write-Host "This script identifies files that need conversion" -ForegroundColor Yellow

$files = Get-ChildItem -Path "examples\penetration" -Filter "*.rs" -Recurse

$needsConversion = @()
foreach ($file in $files) {
    $content = Get-Content $file.FullName -Raw
    if ($content -match "use gun::Gun;" -and -not ($content -match "GunOptions")) {
        if (-not ($content -match "client1|client2|two.*client")) {
            $needsConversion += $file
        }
    }
}

Write-Host "`nFound $($needsConversion.Count) files that likely need conversion" -ForegroundColor Cyan
$needsConversion | ForEach-Object { Write-Host "  $($_.Name)" -ForegroundColor Gray }

Write-Host "`nPattern to apply:" -ForegroundColor Yellow
Write-Host "1. Change: use gun::Gun; -> use gun::{Gun, GunOptions};" -ForegroundColor White
Write-Host "2. Change: let gun = Gun::new(); -> Create client1 and client2 with GunOptions" -ForegroundColor White
Write-Host "3. Add: const RELAY_URL: &str = `"$relayUrl`";" -ForegroundColor White
Write-Host "4. Split operations: client1 puts, client2 reads/verifies" -ForegroundColor White
Write-Host "5. Add synchronization delays" -ForegroundColor White

