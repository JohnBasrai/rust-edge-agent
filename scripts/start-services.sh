#!/usr/bin/env bash
set -euo pipefail

NAT_PORT=4222
MQTT_PORT=1883

echo "Starting NATS broker on localhost:${NAT_PORT}"
docker run -d --name nats_svc -p ${NAT_PORT}:${NAT_PORT} nats:latest

echo "Starting mosquitto broker on localhost:MQTT_PORT"
docker run -d --name mqtt_svc -p ${MQTT_PORT}:${MQTT_PORT} eclipse-mosquitto
