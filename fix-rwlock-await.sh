#!/bin/bash

# Fix all .read() and .write() calls to add .await
for file in systems/logic/src/*.rs; do
    echo "Processing $file..."
    
    # Replace .read() with .read().await
    sed -i 's/\.read()/\.read().await/g' "$file"
    
    # Replace .write() with .write().await
    sed -i 's/\.write()/\.write().await/g' "$file"
    
    # Fix double awaits if any were created
    sed -i 's/\.await\.await/.await/g' "$file"
done

echo "Done fixing RwLock await calls"