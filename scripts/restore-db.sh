#!/bin/bash
# SpacetimeDB full restore script
# Restores: data + ECDSA keys + admin token from a single backup
#
# Usage:
#   bash scripts/restore-db.sh <backup.tar.gz>   # Full restore
#   bash scripts/restore-db.sh --list             # List available backups
#   bash scripts/restore-db.sh --latest           # Restore from latest backup

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."
BACKUP_DIR="$PROJECT_DIR/scripts/backups"
CONTAINER="discord-clone-spacetimedb-nuxt-spacetimedb-1"

if [ "$1" = "--list" ]; then
  echo "Available backups:"
  ls -lht "$BACKUP_DIR"/stdb-*.tar.gz 2>/dev/null | awk '{print $NF, $5}' | head -20
  echo ""
  echo "Latest: $(ls -t "$BACKUP_DIR"/stdb-*.tar.gz 2>/dev/null | head -1)"
  exit 0
fi

BACKUP_FILE="$1"
if [ "$1" = "--latest" ]; then
  BACKUP_FILE=$(ls -t "$BACKUP_DIR"/stdb-*.tar.gz 2>/dev/null | head -1)
  if [ -z "$BACKUP_FILE" ]; then
    echo "Error: No backups found in $BACKUP_DIR"
    exit 1
  fi
  echo "Using latest backup: $BACKUP_FILE"
fi

if [ -z "$BACKUP_FILE" ]; then
  echo "Usage: bash scripts/restore-db.sh <backup.tar.gz>"
  echo "       bash scripts/restore-db.sh --list"
  echo "       bash scripts/restore-db.sh --latest"
  exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
  echo "Error: Backup file not found: $BACKUP_FILE"
  exit 1
fi

# Verify backup integrity
echo "[1/7] Verifying backup integrity..."
if ! tar tzf "$BACKUP_FILE" > /dev/null 2>&1; then
  echo "Error: Backup file is corrupted"
  exit 1
fi
echo "  OK"

# Extract to temp
echo "[2/7] Extracting backup..."
TMPDIR="/tmp/stdb-restore-$$"
mkdir -p "$TMPDIR"
tar xzf "$BACKUP_FILE" -C "$TMPDIR"

# Check what's in the backup
HAS_KEYS=false
HAS_TOKEN=false
[ -f "$TMPDIR/keys/id_ecdsa" ] && HAS_KEYS=true
[ -f "$TMPDIR/env.publish" ] && HAS_TOKEN=true
echo "  Data: $(du -sh "$TMPDIR/data" 2>/dev/null | cut -f1 || echo 'missing!')"
echo "  Keys: $HAS_KEYS"
echo "  Token: $HAS_TOKEN"

if [ ! -d "$TMPDIR/data" ]; then
  echo "Error: Backup does not contain data directory"
  rm -rf "$TMPDIR"
  exit 1
fi

# Safety backup
echo "[3/7] Creating safety backup of current data..."
SAFETY_BACKUP="$BACKUP_DIR/stdb-pre-restore-$(date +%Y%m%d-%H%M%S).tar.gz"
docker cp "$CONTAINER:/home/spacetime/.local/share/spacetime/data" "/tmp/stdb-safety-tmp" 2>/dev/null || true
if [ -d "/tmp/stdb-safety-tmp" ]; then
  tar czf "$SAFETY_BACKUP" -C /tmp/stdb-safety-tmp . 2>/dev/null
  rm -rf /tmp/stdb-safety-tmp
  echo "  Safety backup: $SAFETY_BACKUP"
else
  echo "  Warning: Could not create safety backup"
fi

# Stop SpacetimeDB
echo "[4/7] Stopping SpacetimeDB..."
cd "$PROJECT_DIR"
docker compose stop spacetimedb 2>&1 | grep -v "^$"

# Restore data volume
echo "[5/7] Restoring data..."
docker run --rm \
  -v discord-clone-spacetimedb-nuxt_stdb-data:/data \
  -v "$TMPDIR/data":/backup:ro \
  alpine sh -c "rm -rf /data/* && cp -a /backup/* /data/ && chown -R 1000:1000 /data && find /data -name '*.lock' -delete"

# Restore ECDSA keys
if [ "$HAS_KEYS" = true ]; then
  echo "[5b] Restoring ECDSA keys..."
  docker run --rm \
    -v discord-clone-spacetimedb-nuxt_stdb-keys:/keys \
    -v "$TMPDIR/keys":/backup-keys:ro \
    alpine sh -c "cp /backup-keys/id_ecdsa /keys/id_ecdsa && cp /backup-keys/id_ecdsa.pub /keys/id_ecdsa.pub && chown 1000:1000 /keys/*"
  # Also save to project directory
  cp "$TMPDIR/keys/id_ecdsa" "$PROJECT_DIR/.spacetime-id_ecdsa"
  cp "$TMPDIR/keys/id_ecdsa.pub" "$PROJECT_DIR/.spacetime-id_ecdsa.pub"
fi

# Restore admin token
if [ "$HAS_TOKEN" = true ]; then
  echo "[5c] Restoring admin token..."
  cp "$TMPDIR/env.publish" "$PROJECT_DIR/.env.publish"
fi

# Cleanup temp
rm -rf "$TMPDIR"

# Start SpacetimeDB
echo "[6/7] Starting SpacetimeDB..."
docker compose start spacetimedb 2>&1 | grep -v "^$"

# Wait for health
echo "[7/7] Waiting for SpacetimeDB..."
for i in $(seq 1 30); do
  STATUS=$(docker inspect --format='{{.State.Health.Status}}' "$CONTAINER" 2>/dev/null || echo "unknown")
  if [ "$STATUS" = "healthy" ]; then
    echo "  SpacetimeDB is healthy!"
    break
  fi
  if [ "$i" -eq 30 ]; then
    echo "  WARNING: Not healthy after 30s — check logs"
    docker compose logs spacetimedb --tail 10
    exit 1
  fi
  sleep 1
done

# Verify data
echo ""
echo "=== Restored Data ==="
docker compose exec spacetimedb spacetime sql discord-clone -s http://127.0.0.1:3000 "SELECT username FROM user_profile" 2>&1 | grep '"'
echo ""

# Test publish still works
echo "=== Publish Test ==="
if [ -f "$PROJECT_DIR/.env.publish" ]; then
  source "$PROJECT_DIR/.env.publish"
  docker compose exec spacetimedb spacetime login --token "$SPACETIMEDB_ADMIN_TOKEN" 2>/dev/null
  echo "  Admin login: OK"
else
  echo "  Warning: No .env.publish — publish may not work"
fi

echo ""
echo "=== Restore Complete ==="
[ -n "$SAFETY_BACKUP" ] && echo "Safety backup at: $SAFETY_BACKUP"
