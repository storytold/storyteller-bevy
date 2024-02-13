#!/bin/bash

set -euxo pipefail

# Install dependencies
rustup target add wasm32-unknown-unknown
npm ci

# Build projects
npx nx run studio:build:web
npx nx run studio-frontend:build:production
