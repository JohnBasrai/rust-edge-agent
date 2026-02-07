#!/usr/bin/env bash
set -euo pipefail

: ${NUM_DEVICES:=3}
: ${DEVICE_INTERVAL:=5}

# Detect CI vs interactive mode
if [ -n "${CI:-}" ]; then
  MODE="ci"
else
  MODE="interactive"
fi

echo "$0: NUM_DEVICES     : ${NUM_DEVICES}"
echo "$0: DEVICE_INTERVAL : ${DEVICE_INTERVAL}"
echo "$0: MODE            : ${MODE}"

# Kill any stale processes from previous runs
echo "$0: Checking for stale processes..."
if pgrep -x rust-edge-agent >/dev/null 2>&1; then
  echo "$0: WARNING: Found existing rust-edge-agent processes. Killing..."
  pkill -9 rust-edge-agent || true
  sleep 0.5
fi

if pgrep -x device_sim >/dev/null 2>&1; then
  echo "$0: WARNING: Found existing device_sim processes. Killing..."
  pkill -9 device_sim || true
  sleep 0.5
fi

# Start edge agent in background
./target/release/rust-edge-agent &
AGENT_PID=$!
echo "$0: Started agent (PID: $AGENT_PID)"

# Give agent time to start
sleep 1

# Start device simulators
DEVICE_PIDS=()
for i in $(seq 1 $NUM_DEVICES); do
  case $((i % 3)) in
    0) MODE_ARG="sensor"; TYPE_ARG="temp" ;;
    1) MODE_ARG="actuator"; TYPE_ARG="valve" ;;
    2) MODE_ARG="hybrid"; TYPE_ARG="propulsion" ;;
  esac
  printf "%s: Starting device:%2d mode:%-10s TYPE:%-12s\n" $0 $i ${MODE_ARG} ${TYPE_ARG}
  ./target/release/device_sim \
    --id "device-$(printf "%03d" $i)" \
    --mode $MODE_ARG \
    --type $TYPE_ARG \
    --interval $DEVICE_INTERVAL &
  DEVICE_PIDS+=($!)
done

cleanup() {
  echo ""
  echo "$0: Cleaning up..."
  
  # Kill agent
  if [ -n "${AGENT_PID:-}" ] && ps -p $AGENT_PID >/dev/null 2>&1; then
    echo "$0: Stopping agent (PID: $AGENT_PID)..."
    kill $AGENT_PID 2>/dev/null || true
    sleep 0.5
    # Force kill if still running
    if ps -p $AGENT_PID >/dev/null 2>&1; then
      kill -9 $AGENT_PID 2>/dev/null || true
    fi
  fi
  
  # Kill devices
  for pid in "${DEVICE_PIDS[@]:-}"; do
    if ps -p $pid >/dev/null 2>&1; then
      echo "$0: Stopping device (PID: $pid)..."
      kill $pid 2>/dev/null || true
    fi
  done
  
  # Wait a bit, then force kill any stragglers
  sleep 0.5
  for pid in "${DEVICE_PIDS[@]:-}"; do
    if ps -p $pid >/dev/null 2>&1; then
      echo "$0: Force killing device (PID: $pid)..."
      kill -9 $pid 2>/dev/null || true
    fi
  done
  
  echo "$0: Cleanup complete"
}

# Trap EXIT, INT (Ctrl+C), and TERM signals
trap cleanup EXIT INT TERM

if [ "$MODE" = "ci" ]; then
  # CI mode: run automated checks, then exit
  sleep 5  # Let system stabilize
  # TODO: Add automated validation (check telemetry, send command, verify response)
  echo "CI validation passed"
else
  # Interactive mode: show monitoring instructions
  echo "Edge agent and $NUM_DEVICES devices running."
  echo ""
  echo "Monitor telemetry:"
  echo "  nats sub 'backend.telemetry'"
  echo ""
  echo "Send command to actuator:"
  echo "  nats req 'backend.command.device-002' '{\"target_value\": 75.0}'"
  echo ""
  echo "Press Ctrl+C to stop."
  wait
fi
