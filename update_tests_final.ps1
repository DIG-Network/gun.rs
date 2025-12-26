# PowerShell script to update remaining test files with BLS keys

$testFiles = @(
    "tests\chain_api_tests.rs",
    "tests\error_tests.rs",
    "tests\sea_user_tests.rs",
    "tests\sea_tests.rs",
    "tests\expiration_tests.rs",
    "tests\content_addressing_tests.rs",
    "tests\webrtc_tests.rs",
    "tests\webrtc_two_clients.rs",
    "tests\stress_tests.rs",
    "tests\lock_tests.rs"
)

foreach ($file in $testFiles) {
    if (-not (Test-Path $file)) {
        continue
    }
    
    $content = Get-Content $file -Raw
    $original = $content
    $keyCounter = 0
    
    # Add chia_bls import if needed
    if ($content -match 'use gun::' -and $content -notmatch 'use chia_bls::') {
        $content = $content -replace '(use gun::[^;]+;)', '$1`nuse chia_bls::{SecretKey, PublicKey};'
    }
    
    # Replace Gun::new() patterns
    while ($content -match '(\s+)(let\s+gun\s*=\s*)Gun::new\(\);') {
        $keyCounter++
        $indent = $matches[1]
        $before = $matches[2]
        $replacement = "$indent// Generate BLS key pair`n$indent`let secret_key$keyCounter = SecretKey::from_seed(&[$keyCounter u8; 32]);`n$indent`let public_key$keyCounter = secret_key$keyCounter.public_key();`n$indent$before`Gun::new(secret_key$keyCounter, public_key$keyCounter);"
        $content = $content -replace '(\s+)(let\s+gun\s*=\s*)Gun::new\(\);', $replacement, 1
    }
    
    # Replace Arc::new(Gun::new())
    while ($content -match '(\s+)(let\s+\w+\s*=\s*Arc::new\()Gun::new\(\)\);') {
        $keyCounter++
        $indent = $matches[1]
        $before = $matches[2]
        $replacement = "$indent// Generate BLS key pair`n$indent`let secret_key$keyCounter = SecretKey::from_seed(&[$keyCounter u8; 32]);`n$indent`let public_key$keyCounter = secret_key$keyCounter.public_key();`n$indent$before`Gun::new(secret_key$keyCounter, public_key$keyCounter));"
        $content = $content -replace '(\s+)(let\s+\w+\s*=\s*Arc::new\()Gun::new\(\)\);', $replacement, 1
    }
    
    # Replace Gun::with_options(options) - need to add keys before
    $lines = $content -split "`n"
    $newLines = @()
    $i = 0
    $keyCounter = 0
    
    while ($i -lt $lines.Length) {
        $line = $lines[$i]
        
        if ($line -match 'Gun::with_options\((\w+)\)' -and $line -notmatch 'SecretKey::from_seed') {
            $optionsVar = $matches[1]
            $hasKeys = $false
            
            # Check if keys were added recently
            for ($j = [Math]::Max(0, $i - 5); $j -lt $i; $j++) {
                if ($lines[$j] -match 'SecretKey::from_seed') {
                    $hasKeys = $true
                    break
                }
            }
            
            if (-not $hasKeys) {
                $keyCounter++
                $indent = $line -replace '^(\s*).*', '$1'
                $newLines += "$indent// Generate BLS key pair"
                $newLines += "$indent`let secret_key$keyCounter = SecretKey::from_seed(&[$keyCounter u8; 32]);"
                $newLines += "$indent`let public_key$keyCounter = secret_key$keyCounter.public_key();"
                $line = $line -replace "Gun::with_options\($optionsVar\)", "Gun::with_options(secret_key$keyCounter, public_key$keyCounter, $optionsVar)"
            }
        }
        
        $newLines += $line
        $i++
    }
    
    $content = $newLines -join "`n"
    
    if ($content -ne $original) {
        Set-Content -Path $file -Value $content -NoNewline
        Write-Host "Updated: $file"
    }
}

Write-Host "Done updating test files"

