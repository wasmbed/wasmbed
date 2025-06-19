#!/bin/sh
set -euo pipefail

ELF_FILE="$1"
QEMU_SERIAL_SOCK=$(mktemp)

qemu-system-riscv32                                \
    -nographic                                     \
    -monitor none                                  \
    -machine sifive_e,revb=true                    \
    -d guest_errors                                \
    -serial unix:"$QEMU_SERIAL_SOCK",server,nowait \
    -kernel "$ELF_FILE" &

QEMU_PID=$!

# Wait for socket readiness
for i in $(seq 1 50); do
    if socat -u /dev/null "UNIX-CONNECT:$QEMU_SERIAL_SOCK" 2>/dev/null; then
        break
    fi
    sleep 0.05
done

# Connect to socket and read-only forward output
socat -u "UNIX-CONNECT:$QEMU_SERIAL_SOCK" - | defmt-print -e "$ELF_FILE"

kill "$QEMU_PID"
rm -f "$QEMU_SERIAL_SOCK"
