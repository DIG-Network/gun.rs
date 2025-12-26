#!/usr/bin/env python3
"""
Script to update Gun examples and tests to use BLS key pairs.
"""
import re
import os
from pathlib import Path

def update_file(file_path):
    """Update a single file to use BLS keys."""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    updated = False
    
    # Add chia_bls import if not present and file uses Gun
    if 'use gun::' in content or 'use gun::' in content:
        if 'use chia_bls::' not in content:
            # Find the last use statement
            use_pattern = r'(use [^;]+;)'
            matches = list(re.finditer(use_pattern, content))
            if matches:
                last_use = matches[-1]
                insert_pos = last_use.end()
                content = content[:insert_pos] + '\nuse chia_bls::{SecretKey, PublicKey};' + content[insert_pos:]
                updated = True
    
    # Replace Gun::new() with key generation + Gun::new(secret_key, public_key)
    if 'Gun::new()' in content:
        # Find the line with Gun::new()
        pattern = r'(\s+)(let\s+gun\s*=\s*)Gun::new\(\);'
        replacement = r'\1// Generate BLS key pair\n\1let secret_key = SecretKey::from_seed(&[0u8; 32]);\n\1let public_key = secret_key.public_key();\n\1\2Gun::new(secret_key, public_key);'
        new_content = re.sub(pattern, replacement, content)
        if new_content != content:
            content = new_content
            updated = True
        
        # Also handle Arc::new(Gun::new())
        pattern2 = r'(\s+)(let\s+\w+\s*=\s*Arc::new\()Gun::new\(\)\);'
        replacement2 = r'\1// Generate BLS key pair\n\1let secret_key = SecretKey::from_seed(&[0u8; 32]);\n\1let public_key = secret_key.public_key();\n\1\2Gun::new(secret_key, public_key));'
        new_content = re.sub(pattern2, replacement2, content)
        if new_content != content:
            content = new_content
            updated = True
    
    # Replace Gun::with_options(options) with key generation + Gun::with_options(secret_key, public_key, options)
    # This is more complex - need to handle different patterns
    if 'Gun::with_options(' in content:
        # Pattern 1: match Gun::with_options(options).await
        pattern1 = r'(\s+)(let\s+secret_key\d*\s*=\s*SecretKey::from_seed\([^)]+\);\s*\n\s*let\s+public_key\d*\s*=\s*secret_key\d*\.public_key\(\);)?\s*(let\s+\w+\s*=\s*match\s+)?Gun::with_options\((\w+)\)\.await'
        
        # First, let's find all instances and update them with unique key names
        lines = content.split('\n')
        new_lines = []
        i = 0
        key_counter = 1
        
        while i < len(lines):
            line = lines[i]
            
            # Check if this line has Gun::with_options(options)
            if 'Gun::with_options(' in line and 'SecretKey::from_seed' not in '\n'.join(lines[max(0, i-5):i]):
                # Check if we need to add key generation before this line
                indent = len(line) - len(line.lstrip())
                indent_str = ' ' * indent
                
                # Extract the options variable name
                match = re.search(r'Gun::with_options\((\w+)\)', line)
                if match:
                    options_var = match.group(1)
                    
                    # Insert key generation before this line
                    key_gen = f'{indent_str}let secret_key{key_counter} = SecretKey::from_seed(&[{key_counter}u8; 32]);\n'
                    key_gen += f'{indent_str}let public_key{key_counter} = secret_key{key_counter}.public_key();\n'
                    new_lines.append(key_gen.rstrip())
                    
                    # Update the line to include keys
                    line = line.replace(
                        f'Gun::with_options({options_var})',
                        f'Gun::with_options(secret_key{key_counter}, public_key{key_counter}, {options_var})'
                    )
                    key_counter += 1
                    updated = True
            
            new_lines.append(line)
            i += 1
        
        if updated:
            content = '\n'.join(new_lines)
    
    if updated and content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    return False

def main():
    """Update all example and test files."""
    base_dir = Path('.')
    
    # Find all .rs files in examples and tests directories
    files_to_update = []
    for pattern in ['examples/**/*.rs', 'tests/**/*.rs']:
        files_to_update.extend(base_dir.glob(pattern))
    
    updated_count = 0
    for file_path in sorted(files_to_update):
        if update_file(str(file_path)):
            print(f"Updated: {file_path}")
            updated_count += 1
    
    print(f"\nTotal files updated: {updated_count}")

if __name__ == '__main__':
    main()

