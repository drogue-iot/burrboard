FROM python:3.10

# add testing repo to pick up bluez 5.62
COPY deploy/testing-source.list /etc/apt/sources.list.d/

RUN apt-get update && apt-get install -t testing -y \
    bluez \
    bluez-meshd

RUN apt-get install -y \
    dbus

RUN rm -rf /var/lib/bluetooth/mesh

COPY deploy/bluetooth-mesh-init.sh /etc/init.d/bluetooth-mesh
COPY deploy/entrypoint.sh .
COPY *.py /app/

RUN pip3 install numpy
RUN apt install -y build-essential libdbus-glib-1-dev libgirepository1.0-dev
RUN pip3 install dbus-python
RUN pip3 install glib
RUN pip3 install pygobject
RUN pip3 install paho-mqtt

CMD ./entrypoint.sh
