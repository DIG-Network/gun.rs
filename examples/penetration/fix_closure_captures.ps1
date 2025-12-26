# Script to identify and report files that need closure capture fixes
$files = @(
    "examples/penetration/03_chain_read/01_once_basic.rs",
    "examples/penetration/03_chain_read/03_once_after_put.rs",
    "examples/penetration/03_chain_read/04_once_nested.rs",
    "examples/penetration/03_chain_read/05_once_soul_refs.rs",
    "examples/penetration/03_chain_read/06_once_concurrent.rs",
    "examples/penetration/05_chain_chaining/03_get_put_once.rs",
    "examples/penetration/12_storage/06_storage_persistence.rs",
    "examples/penetration/16_integration/01_full_workflow.rs"
)

Write-Host "Files that need closure capture fixes:" -ForegroundColor Yellow
$files | ForEach-Object { Write-Host "  $_" }

