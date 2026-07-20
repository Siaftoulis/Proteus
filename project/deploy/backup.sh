#!/bin/bash
# ponytail: daily backup script for Docker volumes
# Usage: ./backup.sh [backup-dir]
set -euo pipefail

BACKUP_DIR="${1:-/tmp/crm-backups}"
DATE=$(date +%Y%m%d-%H%M%S)
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

mkdir -p "$BACKUP_DIR/$DATE"

echo "Backing up auth-server data..."
docker compose -f "$PROJECT_DIR/docker-compose.yml" exec -T auth-server tar czf - -C /app data > "$BACKUP_DIR/$DATE/auth-data.tar.gz"

echo "Backing up license-server data..."
docker compose -f "$PROJECT_DIR/docker-compose.yml" exec -T license-server tar czf - -C /app licenses.db > "$BACKUP_DIR/$DATE/license-data.tar.gz"

# Also copy docker-compose.yml and .env
cp "$PROJECT_DIR/docker-compose.yml" "$BACKUP_DIR/$DATE/"
[ -f "$PROJECT_DIR/.env" ] && cp "$PROJECT_DIR/.env" "$BACKUP_DIR/$DATE/"

echo "Backup saved to $BACKUP_DIR/$DATE"
ls -lh "$BACKUP_DIR/$DATE"
