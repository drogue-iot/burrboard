#!/usr/bin/env bash
set -e

REV=$(git rev-parse --short HEAD)
REVISION=${REV} cargo build --release --features defmt
REVISION=${REV} cargo objcopy --release --features defmt -- -O binary firmware.bin
SZ=$(du -b firmware.bin | cut -f1)
cat <<EOF > firmware.json
{
  "version": "${REV}",
  "size": "${SZ}"
}
EOF
