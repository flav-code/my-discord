#!/bin/bash
set -e

CONTAINER="discord-clone-spacetimedb-nuxt-spacetimedb-1"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Load admin token from .env.publish
if [ -f "$SCRIPT_DIR/../.env.publish" ]; then
  source "$SCRIPT_DIR/../.env.publish"
fi

if [ -z "$SPACETIMEDB_ADMIN_TOKEN" ]; then
  echo "Error: SPACETIMEDB_ADMIN_TOKEN not set. Create .env.publish with the token."
  exit 1
fi

echo "Publishing SpacetimeDB module..."

# Login with persisted admin token
docker exec "$CONTAINER" spacetime login --token "$SPACETIMEDB_ADMIN_TOKEN" 2>/dev/null

# Copy server source into the running container
docker cp "$SCRIPT_DIR/../server/." "$CONTAINER:/tmp/module"

# Build and publish — NEVER use --delete-data
docker exec "$CONTAINER" spacetime publish discord-clone \
  -p /tmp/module \
  -s http://127.0.0.1:3000 \
  -y

echo "Module published successfully!"
