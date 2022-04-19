#!/bin/bash

: "${MESH:=true}"

if [[ "$MESH" == "true" ]]; then
service dbus start
service bluetooth-mesh start
fi

exec "$@"
