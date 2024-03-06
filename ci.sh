#!/bin/bash

set -euxo pipefail

# Install build dependencies
rustup update
rustup default stable
rustup target add wasm32-unknown-unknown

# Emit versions
rustup show
npm version

# Build projects
npx nx run studio:build:web
npx nx run studio-frontend:build:production
