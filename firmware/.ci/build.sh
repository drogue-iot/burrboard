#!/usr/bin/env bash
set -e

pushd application
REV=$(git rev-parse --short HEAD)
REVISION=${REV} cargo build --release --features defmt,gatt
REVISION=${REV} cargo objcopy --release --features defmt,gatt -- -O binary firmware.bin
SZ=$(du -b firmware.bin | cut -f1)
cat <<EOF > firmware.json
{
  "version": "${REV}",
  "size": "${SZ}"
}
EOF
popd
