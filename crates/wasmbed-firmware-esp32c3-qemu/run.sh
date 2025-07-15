#!/bin/sh
set -euo pipefail

ELF_FILE="$1"
QEMU_SERIAL_SOCK=$(mktemp)

espflash save-image --chip esp32c3 --merge ${ELF_FILE} /tmp/firmware.bin

qemu-system-esp32c3 -nographic -monitor none -icount 3 -machine esp32c3 -drive file=/tmp/firmware.bin,if=mtd,format=raw -d guest_errors -serial unix:"$QEMU_SERIAL_SOCK",server,nowait &


# Wait for socket readiness
for i in $(seq 1 50); do
    if socat -u /dev/null "UNIX-CONNECT:$QEMU_SERIAL_SOCK" 2>/dev/null; then
        break
    fi
    sleep 0.05
done

# Connect to socket and read-only forward output
socat -u "UNIX-CONNECT:$QEMU_SERIAL_SOCK" - | defmt-print -e $ELF_FILE


rm -f "$QEMU_SERIAL_SOCK"
