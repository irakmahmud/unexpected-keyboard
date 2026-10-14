#!/bin/bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

export PORT=${PORT:-8080}
export DATABASE_URL=${DATABASE_URL:-"sqlite://$DIR/data/attendance.db"}

echo "=========================================================="
echo "⚡ Starting Smart Attendance System (Pure Rust + Axum)"
echo "Port: $PORT"
echo "Database: $DATABASE_URL"
echo "Binary: $DIR/target/release/attendance-api"
echo "=========================================================="

mkdir -p "$DIR/data"
mkdir -p "$DIR/config"

exec "$DIR/target/release/attendance-api"
