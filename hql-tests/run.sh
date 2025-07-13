#!/bin/bash

## if arg is provided, run that file
if [ $# -eq 1 ]; then
    cargo run --release --bin test -- $1
elif [ "$1" = "batch" ] && [ $# -eq 3 ]; then
    cargo run --release --bin test -- --batch $2 $3
else
    cargo run --release --bin test
fi