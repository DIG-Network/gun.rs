# Script to add timeout wrapper to all .once() calls in penetration examples
# This is a helper script - manual verification recommended

$examplesDir = Join-Path $PSScriptRoot ".."
$files = @(
    "examples\penetration\03_chain_read\05_once_soul_refs.rs",
    "examples\penetration\03_chain_read\06_once_concurrent.rs",
    "examples\penetration\16_integration\01_full_workflow.rs",
    "examples\penetration\12_storage\06_storage_persistence.rs",
    "examples\penetration\02_chain_basic\02_put_primitives.rs",
    "examples\penetration\02_chain_basic\03_put_nested.rs",
    "examples\penetration\05_chain_chaining\11_all_methods.rs"
)

Write-Host "Files that still need timeout updates:" -ForegroundColor Cyan
$files | ForEach-Object { Write-Host "  $_" -ForegroundColor Yellow }

Write-Host "`nNote: These files need manual updates to wrap .once() calls in timeout()" -ForegroundColor Gray
Write-Host "Pattern to apply:" -ForegroundColor Gray
Write-Host "  match client2.get(...).once(...).await {" -ForegroundColor White
Write-Host "  becomes:" -ForegroundColor White
Write-Host "  match timeout(Duration::from_secs(10), client2.get(...).once(...)).await {" -ForegroundColor Green
Write-Host "    Ok(Ok(_)) => { ... }" -ForegroundColor Green
Write-Host "    Ok(Err(e)) => { ... }" -ForegroundColor Green
Write-Host "    Err(_) => { /* timeout */ }" -ForegroundColor Green

