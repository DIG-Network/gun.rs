# Summary script - Files still need timeout updates for remaining .once() calls
# This is just for reference - manual updates are recommended for correctness

Write-Host "=== Timeout Update Status ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Files with timeouts added:" -ForegroundColor Green
Write-Host "  ✓ 01_once_basic.rs" -ForegroundColor Green
Write-Host "  ✓ 02_once_missing.rs" -ForegroundColor Green  
Write-Host "  ✓ 03_once_after_put.rs" -ForegroundColor Green
Write-Host "  ✓ 04_once_nested.rs" -ForegroundColor Green
Write-Host "  ✓ 05_once_soul_refs.rs" -ForegroundColor Green
Write-Host "  ✓ 06_once_concurrent.rs" -ForegroundColor Green
Write-Host "  ✓ 03_get_put_once.rs" -ForegroundColor Green
Write-Host "  ✓ 16_integration/01_full_workflow.rs" -ForegroundColor Green
Write-Host "  ✓ 12_storage/06_storage_persistence.rs" -ForegroundColor Green
Write-Host "  ⚠ 02_put_primitives.rs (1/8 done)" -ForegroundColor Yellow
Write-Host ""
Write-Host "Files still needing updates:" -ForegroundColor Yellow
Write-Host "  - 02_chain_basic/02_put_primitives.rs (7 more .once() calls)" -ForegroundColor Yellow
Write-Host "  - 02_chain_basic/03_put_nested.rs (5 .once() calls)" -ForegroundColor Yellow
Write-Host "  - 05_chain_chaining/11_all_methods.rs (1 .once() call)" -ForegroundColor Yellow
Write-Host ""
Write-Host "Pattern to apply:" -ForegroundColor Cyan
Write-Host "  Replace: match client2.get(...).once(...).await {" -ForegroundColor White
Write-Host "  With:    match timeout(Duration::from_secs(10), client2.get(...).once(...)).await {" -ForegroundColor Green
Write-Host "  Then change: Ok(_) => {...} to Ok(Ok(_)) => {...}" -ForegroundColor Green
Write-Host "  Then change: Err(e) => {...} to Ok(Err(e)) => {...}" -ForegroundColor Green
Write-Host "  Then add:    Err(_) => { /* timeout */ }" -ForegroundColor Green

