#!/bin/bash

service dbus start
service bluetooth-mesh start

# We'll eventually replace this with the python gateway
tail -f /dev/null
