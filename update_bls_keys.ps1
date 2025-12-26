# PowerShell script to update Gun examples and tests to use BLS key pairs

$files = Get-ChildItem -Path . -Include *.rs -Recurse | Where-Object {
    $_.FullName -notmatch '\\target\\' -and
    ($_.FullName -match '\\examples\\' -or $_.FullName -match '\\tests\\')
}

$updatedCount = 0

foreach ($file in $files) {
    $content = Get-Content $file.FullName -Raw
    $originalContent = $content
    $changed = $false
    
    # Add chia_bls import if needed
    if ($content -match 'use gun::' -and $content -notmatch 'use chia_bls::') {
        # Find the last use statement
        if ($content -match '(use [^;]+;)') {
            $lastUse = [regex]::Matches($content, '(use [^;]+;)') | Select-Object -Last 1
            if ($lastUse) {
                $insertPos = $lastUse.Index + $lastUse.Length
                $content = $content.Insert($insertPos, "`nuse chia_bls::{SecretKey, PublicKey};")
                $changed = $true
            }
        }
    }
    
    # Counter for multiple instances in same file
    $keyCounter = 0
    
    # Replace Gun::new() patterns
    while ($content -match '(\s+)(let\s+gun\s*=\s*)Gun::new\(\);') {
        $keyCounter++
        $indent = $matches[1]
        $replacement = "$indent// Generate BLS key pair`n$indent`let secret_key = SecretKey::from_seed(&[$keyCounter u8; 32]);`n$indent`let public_key = secret_key.public_key();`n$indent$($matches[2])Gun::new(secret_key, public_key);"
        $content = $content -replace '(\s+)(let\s+gun\s*=\s*)Gun::new\(\);', $replacement, 1
        $changed = $true
    }
    
    # Replace Arc::new(Gun::new())
    while ($content -match '(\s+)(let\s+\w+\s*=\s*Arc::new\()Gun::new\(\)\);') {
        $keyCounter++
        $indent = $matches[1]
        $replacement = "$indent// Generate BLS key pair`n$indent`let secret_key = SecretKey::from_seed(&[$keyCounter u8; 32]);`n$indent`let public_key = secret_key.public_key();`n$indent$($matches[2])Gun::new(secret_key, public_key));"
        $content = $content -replace '(\s+)(let\s+\w+\s*=\s*Arc::new\()Gun::new\(\)\);', $replacement, 1
        $changed = $true
    }
    
    # Replace Gun::with_options(options) - need to find the options variable first
    # This is more complex - we'll look for the pattern and add keys before it
    $lines = $content -split "`n"
    $newLines = @()
    $i = 0
    
    while ($i -lt $lines.Length) {
        $line = $lines[$i]
        
        # Check if this line has Gun::with_options(options) without keys
        if ($line -match 'Gun::with_options\((\w+)\)' -and $line -notmatch 'SecretKey::from_seed') {
            $optionsVar = $matches[1]
            
            # Check if keys were already added (look back a few lines)
            $hasKeys = $false
            for ($j = [Math]::Max(0, $i - 5); $j -lt $i; $j++) {
                if ($lines[$j] -match 'SecretKey::from_seed') {
                    $hasKeys = $true
                    break
                }
            }
            
            if (-not $hasKeys) {
                $keyCounter++
                $indent = $line -replace '^(\s*).*', '$1'
                
                # Insert key generation before this line
                $newLines += "$indent// Generate BLS key pair"
                $newLines += "$indent`let secret_key$keyCounter = SecretKey::from_seed(&[$keyCounter u8; 32]);"
                $newLines += "$indent`let public_key$keyCounter = secret_key$keyCounter.public_key();"
                
                # Update the line to include keys
                $line = $line -replace "Gun::with_options\($optionsVar\)", "Gun::with_options(secret_key$keyCounter, public_key$keyCounter, $optionsVar)"
                $changed = $true
            }
        }
        
        $newLines += $line
        $i++
    }
    
    if ($changed) {
        $content = $newLines -join "`n"
    }
    
    if ($content -ne $originalContent) {
        Set-Content -Path $file.FullName -Value $content -NoNewline
        Write-Host "Updated: $($file.FullName)"
        $updatedCount++
    }
}

Write-Host "`nTotal files updated: $updatedCount"

