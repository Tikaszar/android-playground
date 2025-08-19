#!/bin/bash

# Fix functions that use .await to be async
for file in systems/logic/src/*.rs; do
    echo "Processing $file..."
    
    # Make functions that contain .await async
    # Match pub fn and fn patterns that contain .await in their body
    
    # Fix simple pub fn that have .await
    perl -i -0pe 's/(pub fn \w+[^{]*\{[^}]*\.await[^}]*\})/my $x = $1; $x =~ s|pub fn|pub async fn|; $x/ge' "$file"
    
    # Fix simple fn that have .await  
    perl -i -0pe 's/((?<!pub )fn \w+[^{]*\{[^}]*\.await[^}]*\})/my $x = $1; $x =~ s|fn |async fn |; $x/ge' "$file"
    
    # Fix multiline functions - more aggressive approach
    # This finds any function signature followed by a body containing .await
    perl -i -0pe 's/(pub fn)(\s+\w+[^{]*\{(?:[^{}]++|(?3))*\.await)/pub async fn$2/gx' "$file"
    perl -i -0pe 's/((?<!pub\s)fn)(\s+\w+[^{]*\{(?:[^{}]++|(?3))*\.await)/async fn$2/gx' "$file"
    
    # Fix impl block methods
    perl -i -0pe 's/(impl[^{]*\{[^}]*)(pub fn)([^{]*\{[^}]*\.await[^}]*\})/$1pub async fn$3/g' "$file"
    perl -i -0pe 's/(impl[^{]*\{[^}]*)(fn)([^{]*\{[^}]*\.await[^}]*\})/$1async fn$3/g' "$file"
    
done

echo "Done fixing async functions"