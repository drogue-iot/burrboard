#!/usr/bin/env bash
set -e

REV=$(git rev-parse --short HEAD)
REVISION=${REV} cargo build --release --features defmt,rev2
REVISION=${REV} cargo objcopy --release --features defmt,rev2 -- -O binary firmware.bin
SZ=$(du -b firmware.bin | cut -f1)
cat <<EOF > firmware.json
{
  "version": "${REV}",
  "size": "${SZ}"
}
EOF
