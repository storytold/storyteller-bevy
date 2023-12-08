#!/bin/bash

set -euxo pipefail

npm ci
npx nx run studio:build:web
npx nx run react-frontend:build:production --output-path dist/studio-frontend
