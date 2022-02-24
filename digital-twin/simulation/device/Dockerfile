FROM registry.access.redhat.com/ubi8/nginx-120:latest

ARG ENDPOINT_URL="wss://mqtt-endpoint-ws-browser-drogue-dev.apps.wonderful.iot-playground.org/mqtt"

ADD device.js index.html ./

RUN true \
    && mv index.html index.html.in \
    && envsubst '${ENDPOINT_URL}' < index.html.in > index.html \
    && rm index.html.in

CMD nginx -g "daemon off;"
