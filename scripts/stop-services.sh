#!/usr/bin/env bash
set -euo pipefail

echo "Stopping NATS broker..."
docker stop nats_svc 2>/dev/null || true
docker rm nats_svc   2>/dev/null || true

echo "Stopping MQTT broker..."
docker stop mqtt_svc 2>/dev/null || true
docker rm mqtt_svc   2>/dev/null || true
