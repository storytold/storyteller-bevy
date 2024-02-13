#!/bin/bash

set -euxo pipefail

# Install dependencies
rustup target add wasm32-unknown-unknown
npm ci

# Run checks
cargo fmt --all -- --check
npx nx affected -t lint test --parallel=1 --base=main --head=HEAD

# Build projects
npx nx run studio:build:web
npx nx run studio-frontend:build:production
