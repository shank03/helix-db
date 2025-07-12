#!/bin/bash

## if arg is provided, run that file
if [ $# -eq 1 ]; then
    cargo run --release --bin test -- $1
else
    cargo run --release 
fi