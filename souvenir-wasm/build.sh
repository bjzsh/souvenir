#!/usr/bin/env bash

set -e

if ! [ -x "$(command -v wasm-pack)" ]; then
    echo "wasm-pack is not installed" >& 2
    exit 1
fi

wasm-pack build -t nodejs -d pkg
