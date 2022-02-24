FROM registry.access.redhat.com/ubi8/nginx-120:latest

ARG ENDPOINT_URL="wss://mqtt-endpoint-ws-browser-drogue-dev.apps.wonderful.iot-playground.org/mqtt"

ADD device.js index.html ./

RUN true \
    && echo "Replacing endpoint: ${ENDPOINT_URL}" \
    && mv device.js device.js.in \
    && envsubst '${ENDPOINT_URL}' < device.js.in > device.js \
    && rm device.js.in

CMD nginx -g "daemon off;"
