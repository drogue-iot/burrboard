#!/bin/bash

service dbus start
service bluetooth-mesh start

exec "$@"
