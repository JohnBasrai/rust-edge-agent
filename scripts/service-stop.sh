#!/usr/bin/env bash
set -euo pipefail

echo "Stopping NATS broker..."
docker stop nats 2>/dev/null || true
docker rm nats 2>/dev/null || true
echo "NATS broker stopped"
