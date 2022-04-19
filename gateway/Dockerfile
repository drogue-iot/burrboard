FROM python:3.9

# add testing repo to pick up bluez 5.62
COPY deploy/testing-source.list /etc/apt/sources.list.d/

RUN apt-get update && apt-get install -t testing -y \
    libatlas-base-dev \
    build-essential libdbus-glib-1-dev libgirepository1.0-dev \
    bluez bluez-meshd dbus

RUN rm -rf /var/lib/bluetooth/mesh

COPY deploy/bluetooth-mesh-init.sh /etc/init.d/bluetooth-mesh
COPY deploy/entrypoint.sh .
COPY app/*.py /app/

# Set environment variables
ENV PYTHONDONTWRITEBYTECODE 1
ENV PYTHONUNBUFFERED 1
ENV PIP_EXTRA_INDEX_URL=https://www.piwheels.org/simple


RUN pip3 install numpy==1.22.0
#RUN apt install -y build-essential libdbus-glib-1-dev libgirepository1.0-dev
RUN pip3 install dbus-python
RUN pip3 install glib
RUN pip3 install pygobject
RUN pip3 install paho-mqtt

RUN mkdir logs/

ENTRYPOINT ["./entrypoint.sh"]
