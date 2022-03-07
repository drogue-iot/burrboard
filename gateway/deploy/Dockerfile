FROM python:3.10-slim

# add testing repo to pick up bluez 5.62
COPY testing-source.list /etc/apt/sources.list.d/

RUN apt-get update && apt-get install -t testing -y \
    bluez \
    bluez-meshd

RUN apt-get install -y \
    dbus

RUN rm -rf /var/lib/bluetooth/mesh
COPY mesh /var/lib/bluetooth/mesh
COPY bluetooth-mesh-init.sh /etc/init.d/bluetooth-mesh
COPY entrypoint.sh .

CMD ./entrypoint.sh
