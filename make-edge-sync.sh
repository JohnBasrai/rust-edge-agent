#!/bin/bash

tar cfvz make-edge-sync.gz \
    .cargo \
    Cargo.lock \
    Cargo.toml \
    .gitignore \
    LICENSE \
    README.md \
    docs \
    src
    
