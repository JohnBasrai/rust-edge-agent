#!/bin/bash

tar cfvz make-edge-sync.gz \
    .cargo \
    .github \
    .gitignore \
    CHANGELOG.md \
    Cargo.lock \
    Cargo.toml \
    LICENSE \
    README.md \
    docs \
    src scripts
    
