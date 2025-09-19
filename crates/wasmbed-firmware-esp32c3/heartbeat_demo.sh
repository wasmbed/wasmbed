#!/usr/bin/env bash
set -euo pipefail

# Check if espflash is installed
if ! command -v espflash &> /dev/null; then
    echo "espflash not found. Installing..."
    cargo install espflash --locked
    echo "espflash installed successfully."
fi

# Enter nix develop environment
#nix develop

# Resolve script directory (where heartbeat_demo.sh is located)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Target firmware path (relative to project root)
FIRMWARE_DIR="$SCRIPT_DIR/../crates/wasmbed-firmware-esp32c3"

CURRENT_DIR="$(basename "$(pwd)")"

case "$CURRENT_DIR" in
    wasmbed-firmware-esp32c3)
        echo "Already inside firmware folder."
        ;;
    crates)
        if [ -d "wasmbed-firmware-esp32c3" ]; then
            echo "Entering firmware folder from crates/"
            cd wasmbed-firmware-esp32c3
        else
            echo "No wasmbed-firmware-esp32c3 inside crates/"
            exit 1
        fi
        ;;
    *)
        if [ -d "$FIRMWARE_DIR" ]; then
            echo "Entering firmware folder: $FIRMWARE_DIR"
            cd "$FIRMWARE_DIR"
        else
            echo "Could not locate crates/wasmbed-firmware-esp32c3"
            exit 1
        fi
        ;;
esac

# Ask user for input
read -rp "Enter WiFi SSID: " WIFI_SSID
read -rp "Enter WiFi Password: " WIFI_PASS
read -rp "Enter Gateway IP (e.g. 172.168.1.10): " GATEWAY_IP
read -rp "Enter Gateway Port (e.g. 4423): " GATEWAY_PORT 

# Run the build with the provided environment variables
WIFI_SSID="$WIFI_SSID" \
WIFI_PASS="$WIFI_PASS" \
GATEWAY_IP="$GATEWAY_IP" \
GATEWAY_PORT="$GATEWAY_PORT" \
cargo build --release

# Run with provided environment variables and stream logs
WIFI_SSID="$WIFI_SSID" \
WIFI_PASS="$WIFI_PASS" \
GATEWAY_IP="$GATEWAY_IP" \
GATEWAY_PORT="$GATEWAY_PORT" \
cargo run --release
