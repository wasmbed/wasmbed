#!/bin/sh
set -euo pipefail

ELF_FILE="$1"


espflash save-image --chip esp32c3 --merge ${ELF_FILE} /tmp/firmware.bin

espflash flash --format esp-idf ${ELF_FILE} --monitor

