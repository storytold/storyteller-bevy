#!/bin/bash

set -euxo pipefail

rustup target add wasm32-unknown-unknown

npm ci
cargo fmt --all -- --check
npx nx affected -t lint test --parallel=1
npx nx run studio:build:web
npx nx run studio-frontend:build:production
