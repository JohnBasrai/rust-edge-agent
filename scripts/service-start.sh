#!/usr/bin/env bash
set -euo pipefail

echo "Starting NATS broker..."
docker run -d --name nats -p 4222:4222 nats:latest
echo "NATS broker started on localhost:4222"
