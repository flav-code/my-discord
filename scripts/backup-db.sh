#!/bin/bash
# SpacetimeDB hourly backup script
# Backs up: data directory + server ECDSA keys + admin token
# Keeps backups for 30 days, runs via cron

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/.."
BACKUP_DIR="$PROJECT_DIR/scripts/backups"
CONTAINER="discord-clone-spacetimedb-nuxt-spacetimedb-1"
RETENTION_DAYS=30
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
BACKUP_FILE="$BACKUP_DIR/stdb-$TIMESTAMP.tar.gz"

mkdir -p "$BACKUP_DIR"

# Create temp directory for backup contents
TMPDIR="/tmp/stdb-backup-$TIMESTAMP"
mkdir -p "$TMPDIR"

# 1. Backup SpacetimeDB data
docker cp "$CONTAINER:/home/spacetime/.local/share/spacetime/data" "$TMPDIR/data" 2>/dev/null
if [ $? -ne 0 ]; then
  echo "$(date): BACKUP FAILED - container not running" >> "$BACKUP_DIR/backup.log"
  rm -rf "$TMPDIR"
  exit 1
fi

# 2. Backup ECDSA keys (critical for ownership)
mkdir -p "$TMPDIR/keys"
if [ -f "$PROJECT_DIR/.spacetime-id_ecdsa" ]; then
  cp "$PROJECT_DIR/.spacetime-id_ecdsa" "$TMPDIR/keys/id_ecdsa"
  cp "$PROJECT_DIR/.spacetime-id_ecdsa.pub" "$TMPDIR/keys/id_ecdsa.pub"
else
  docker cp "$CONTAINER:/etc/spacetimedb/id_ecdsa" "$TMPDIR/keys/id_ecdsa" 2>/dev/null || true
  docker cp "$CONTAINER:/etc/spacetimedb/id_ecdsa.pub" "$TMPDIR/keys/id_ecdsa.pub" 2>/dev/null || true
fi

# 3. Backup admin token
if [ -f "$PROJECT_DIR/.env.publish" ]; then
  cp "$PROJECT_DIR/.env.publish" "$TMPDIR/env.publish"
fi

# Create tar.gz
tar czf "$BACKUP_FILE" -C "$TMPDIR" . 2>/dev/null
rm -rf "$TMPDIR"

# Verify backup integrity
if ! tar tzf "$BACKUP_FILE" > /dev/null 2>&1; then
  echo "$(date): BACKUP CORRUPTED - $BACKUP_FILE" >> "$BACKUP_DIR/backup.log"
  rm -f "$BACKUP_FILE"
  exit 1
fi

# Reject suspiciously small backups (< 1KB)
BYTE_SIZE=$(stat -c%s "$BACKUP_FILE" 2>/dev/null || stat -f%z "$BACKUP_FILE" 2>/dev/null)
if [ "$BYTE_SIZE" -lt 1024 ]; then
  echo "$(date): BACKUP TOO SMALL ($BYTE_SIZE bytes) - $BACKUP_FILE" >> "$BACKUP_DIR/backup.log"
  rm -f "$BACKUP_FILE"
  exit 1
fi

SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
echo "$(date): OK - $BACKUP_FILE ($SIZE)" >> "$BACKUP_DIR/backup.log"

# Delete backups older than 30 days
find "$BACKUP_DIR" -name "stdb-*.tar.gz" -mtime +$RETENTION_DAYS -delete

# Keep log file trimmed (last 1000 lines)
tail -1000 "$BACKUP_DIR/backup.log" > "$BACKUP_DIR/backup.log.tmp" && mv "$BACKUP_DIR/backup.log.tmp" "$BACKUP_DIR/backup.log"
