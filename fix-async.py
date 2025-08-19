#!/usr/bin/env python3
import re
import sys
from pathlib import Path

def fix_async_functions(content):
    """Fix functions that contain .await to be async"""
    
    # Pattern to match function signatures
    fn_pattern = r'(\s*)(pub\s+)?(fn\s+\w+(?:<[^>]+>)?(?:\([^)]*\))(?:\s*->\s*[^{]+)?)\s*\{'
    
    lines = content.split('\n')
    result = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Check if this line starts a function
        match = re.match(fn_pattern, line)
        if match:
            indent = match.group(1)
            pub = match.group(2) or ''
            signature = match.group(3)
            
            # Find the end of this function
            brace_count = 1
            fn_lines = [line]
            j = i + 1
            fn_body = ""
            
            while j < len(lines) and brace_count > 0:
                fn_lines.append(lines[j])
                fn_body += lines[j] + "\n"
                brace_count += lines[j].count('{') - lines[j].count('}')
                j += 1
            
            # Check if function body contains .await
            if '.await' in fn_body:
                # Make it async if not already
                if 'async fn' not in line:
                    line = line.replace('fn ', 'async fn ', 1)
                    fn_lines[0] = line
            
            result.extend(fn_lines)
            i = j
        else:
            result.append(line)
            i += 1
    
    return '\n'.join(result)

def main():
    # Process all .rs files in systems/logic/src
    logic_dir = Path('/data/data/com.termux/files/home/android-playground/systems/logic/src')
    
    for rust_file in logic_dir.glob('*.rs'):
        print(f"Processing {rust_file.name}...")
        
        with open(rust_file, 'r') as f:
            content = f.read()
        
        fixed_content = fix_async_functions(content)
        
        with open(rust_file, 'w') as f:
            f.write(fixed_content)
    
    print("Done fixing async functions")

if __name__ == '__main__':
    main()