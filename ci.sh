#!/bin/bash

set -euxo pipefail

# Install dependencies
# - WASM rustc target
rustup target add wasm32-unknown-unknown
# - Bevy Linux dependencies
sudo apt-get update
sudo apt-get install --no-install-recommends libasound2-dev liibudev-dev
# - JS dependencies
npm ci

# Run checks
cargo fmt --all -- --check
npx nx affected -t lint test --parallel=1 --base=main --head=HEAD

# Build projects
npx nx run studio:build:web
npx nx run studio-frontend:build:production
