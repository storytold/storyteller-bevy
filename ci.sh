#!/bin/bash

set -euxo pipefail

# Install dependencies
# - WASM rustc target
rustup target add wasm32-unknown-unknown
# - Bevy Linux dependencies
brew install alsa-lib
# - JS dependencies
npm ci

# Run checks
cargo fmt --all -- --check
npx nx affected -t lint test --parallel=1 --base=main --head=HEAD

# Build projects
npx nx run studio:build:web
npx nx run studio-frontend:build:production
