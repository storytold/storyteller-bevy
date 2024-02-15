#!/bin/bash

set -euxo pipefail

rustup target add wasm32-unknown-unknown

npm ci
npx nx run studio:build:web
npx nx run studio-frontend:build:production

