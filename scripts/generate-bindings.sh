#!/bin/bash
set -e

CONTAINER="discord-clone-spacetimedb-nuxt-spacetimedb-1"
OUT_DIR="$(dirname "$0")/../client/module_bindings"

echo "Generating TypeScript bindings..."
mkdir -p "$OUT_DIR"

# Copy server source into the running container
docker cp "$(dirname "$0")/../server/." "$CONTAINER:/tmp/module"

# Clean previous bindings inside container
docker exec "$CONTAINER" rm -rf /tmp/bindings
docker exec "$CONTAINER" mkdir -p /tmp/bindings

# Generate bindings inside the container
docker exec "$CONTAINER" spacetime generate \
  --lang typescript \
  --out-dir /tmp/bindings \
  --module-path /tmp/module \
  -y

# Copy bindings back out
docker cp "$CONTAINER:/tmp/bindings/." "$OUT_DIR/"

echo "TypeScript bindings generated at client/module_bindings/"
ls -la "$OUT_DIR/"
