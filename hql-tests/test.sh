#!/bin/bash

# Check if file number argument is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <file_number>"
    exit 1
fi

file_num=$1

# Validate input is a number between 1 and 100
if ! [[ "$file_num" =~ ^[0-9]+$ ]] || [ "$file_num" -lt 1 ] || [ "$file_num" -gt 100 ]; then
    echo "Error: Please provide a number between 1 and 100"
    exit 1
fi

folder="file$file_num"
if [ -d "$folder" ]; then
    if ! helix compile --path "$(pwd)/$folder" --output "$(pwd)/$folder" --gen rs; then
        echo "Error: Helix compilation failed"
        exit 1
    fi
    # copy output to helix-container/src/queries.rs
    cp "$(pwd)/$folder/queries.rs" "../helix-container/src/queries.rs"
    # check rust
    cd "../../helix-container"
    cargo check
else
    echo "Error: Directory $folder does not exist"
    exit 1
fi
