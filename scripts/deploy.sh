#!/bin/bash
# Production deployment script
# Usage:
#   bash scripts/deploy.sh                    # Full deploy (module + all services)
#   bash scripts/deploy.sh --skip-module      # Skip module publish (frontend only)
#   bash scripts/deploy.sh --skip-bot         # Skip bot rebuild

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."
cd "$PROJECT_DIR"

SKIP_MODULE=false
SKIP_BOT=false

for arg in "$@"; do
  case $arg in
    --skip-module) SKIP_MODULE=true ;;
    --skip-bot) SKIP_BOT=true ;;
  esac
done

echo "=== Production Deploy ==="
echo ""

# Step 1: Safety backup
echo "[1/6] Creating safety backup..."
bash scripts/backup-db.sh
echo "  Done"

# Step 2: Publish module (if not skipped)
if [ "$SKIP_MODULE" = false ]; then
  echo "[2/6] Publishing SpacetimeDB module..."
  bash scripts/publish-module.sh
  echo ""
  echo "[3/6] Regenerating TypeScript bindings..."
  bash scripts/generate-bindings.sh > /dev/null 2>&1
  echo "  Done"
else
  echo "[2/6] Skipping module publish (--skip-module)"
  echo "[3/6] Skipping bindings generation"
fi

# Step 3: Build Docker images
echo "[4/6] Building Docker images..."
BUILD_TARGETS="nuxt-prod api"
if [ "$SKIP_BOT" = false ]; then
  BUILD_TARGETS="$BUILD_TARGETS bot"
fi
docker compose build $BUILD_TARGETS 2>&1 | tail -5
echo "  Done"

# Step 4: Restart services
echo "[5/6] Restarting services..."
docker compose up -d $BUILD_TARGETS 2>&1 | grep -v "^$"
sleep 5

# Step 5: Health checks
echo "[6/6] Verifying deployment..."
echo ""

ALL_OK=true
for svc in spacetimedb nuxt-prod api; do
  STATUS=$(docker compose ps "$svc" --format "{{.Status}}" 2>/dev/null | head -1)
  if echo "$STATUS" | grep -qi "up"; then
    echo "  ✓ $svc: $STATUS"
  else
    echo "  ✗ $svc: $STATUS"
    ALL_OK=false
  fi
done

if [ "$SKIP_BOT" = false ]; then
  BOT_STATUS=$(docker compose ps bot --format "{{.Status}}" 2>/dev/null | head -1)
  if echo "$BOT_STATUS" | grep -qi "up"; then
    echo "  ✓ bot: $BOT_STATUS"
  else
    echo "  ✗ bot: $BOT_STATUS"
    ALL_OK=false
  fi
fi

echo ""
if [ "$ALL_OK" = true ]; then
  echo "=== Deploy successful! ==="
else
  echo "=== WARNING: Some services may have issues ==="
  echo "Check logs: docker compose logs --tail 20"
fi
