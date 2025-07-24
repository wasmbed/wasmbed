#!/bin/sh
set -euo pipefail

ELF_FILE="$1"
QEMU_SERIAL_SOCK=$(mktemp)

# Create firmware image
espflash save-image --chip esp32c3 --merge "${ELF_FILE}" /tmp/firmware.bin

# Start QEMU
qemu-system-esp32c3 \
    -nographic \
    -monitor none \
    -icount 3 \
    -machine esp32c3 \
    -drive file=/tmp/firmware.bin,if=mtd,format=raw \
    -serial unix:"$QEMU_SERIAL_SOCK",server,nowait &

QEMU_PID=$!

# Wait for socket readiness
for i in $(seq 1 50); do
    if socat -u /dev/null "UNIX-CONNECT:$QEMU_SERIAL_SOCK" 2>/dev/null; then
        break
    fi
    sleep 0.05
done

# Connect to the socket, forward to defmt-print, and also log in real-time
socat -u "UNIX-CONNECT:$QEMU_SERIAL_SOCK" - \
