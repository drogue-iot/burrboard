FROM python:3.10-slim

# add testing repo to pick up bluez 5.62
COPY testing-source.list /etc/apt/sources.list.d/

RUN apt-get update && apt-get install -y \
    systemd-sysv \
    dbus

RUN apt-get install -t testing -y \
    bluez \
    bluez-meshd

RUN rm -rf /var/lib/bluetooth/mesh
COPY mesh /var/lib/bluetooth/mesh

# Enable the services
RUN systemctl disable bluetooth.service && \
    systemctl enable bluetooth-mesh.service

CMD [ "/sbin/init" ]
