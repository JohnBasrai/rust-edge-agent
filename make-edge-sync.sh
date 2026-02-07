#!/bin/bash

tar cfvz edge-sync.gz \
    .cargo \
    .github \
    .gitignore \
    CHANGELOG.md \
    CONTRIBUTING.md \
    Cargo.lock \
    Cargo.toml \
    LICENSE \
    README.md \
    docs \
    src scripts
    
