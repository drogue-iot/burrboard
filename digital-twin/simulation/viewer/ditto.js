const accessToken = "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJKR2VRRG01Szlpc1l3emFSTmdXME4zVU91UUlGWkFfZVFFc0QxREdNUzY0In0.eyJleHAiOjE2NDI2MDkwNzQsImlhdCI6MTY0MjYwODc3NCwiYXV0aF90aW1lIjoxNjQyMDg5NDY4LCJqdGkiOiJkZTlhOGY5OS1hM2UyLTQ2YjktOTE1My0xOGEyNzZiNDAxMDgiLCJpc3MiOiJodHRwczovL3Nzby1kcm9ndWUtZGV2LmFwcHMud29uZGVyZnVsLmlvdC1wbGF5Z3JvdW5kLm9yZy9hdXRoL3JlYWxtcy9kcm9ndWUiLCJhdWQiOlsiZHJvZ3VlIiwiYWNjb3VudCJdLCJzdWIiOiJmNzlkZDUyMC03MDdmLTQ0NGItYmI1MC01NTNmODE0MzVjZGMiLCJ0eXAiOiJCZWFyZXIiLCJhenAiOiJkcm9ndWUiLCJzZXNzaW9uX3N0YXRlIjoiYzg0MTRmMmQtODAzZC00YjBhLTkzNzMtYjkxZmZmMDAxZjQ4IiwiYWNyIjoiMCIsImFsbG93ZWQtb3JpZ2lucyI6WyJodHRwOi8vbG9jYWxob3N0OioiLCJodHRwczovL2NvbnNvbGUtZHJvZ3VlLWRldi5hcHBzLndvbmRlcmZ1bC5pb3QtcGxheWdyb3VuZC5vcmciXSwicmVhbG1fYWNjZXNzIjp7InJvbGVzIjpbImRyb2d1ZS11c2VyIiwib2ZmbGluZV9hY2Nlc3MiLCJ1bWFfYXV0aG9yaXphdGlvbiIsImRlZmF1bHQtcm9sZXMtZHJvZ3VlIl19LCJyZXNvdXJjZV9hY2Nlc3MiOnsiYWNjb3VudCI6eyJyb2xlcyI6WyJtYW5hZ2UtYWNjb3VudCIsIm1hbmFnZS1hY2NvdW50LWxpbmtzIiwidmlldy1wcm9maWxlIl19fSwic2NvcGUiOiJwcm9maWxlIGVtYWlsIG9mZmxpbmVfYWNjZXNzIiwic2lkIjoiYzg0MTRmMmQtODAzZC00YjBhLTkzNzMtYjkxZmZmMDAxZjQ4IiwiZW1haWxfdmVyaWZpZWQiOnRydWUsIm5hbWUiOiJKZW5zIFJlaW1hbm4iLCJwcmVmZXJyZWRfdXNlcm5hbWUiOiJjdHJvbiIsImdpdmVuX25hbWUiOiJKZW5zIiwiZmFtaWx5X25hbWUiOiJSZWltYW5uIiwiZW1haWwiOiJjdHJvbkBkZW50cmFzc2kuZGUifQ.BNACW3OBVdFB1Pt5hB5DKgK40W27C-3pJF3CZWZzQCwUF_RFGIiboT6gv1oYKj0uzQ4wqcEPrK2tv0qDcO192UhIe9ZemZPjMDCX_OzIbB1wGrHgfld5-KbmcG3MIpErZznUSo7z9Ry4OLTTcTmRJg54XFsiOBIDJCgKhxM0pxsYPQYrqNoei0yE1RFK_X8M_ZAhG_9PTDuOMk03V8jNr8VpDp8pDi8bo3bjouIpFi5SPrG8A-Yd-xbhIgV4VF0FOZAm1d4B9tYB64IJ6xG2i_edqYYLwDowDWlrH9dqBzZNQF6YShezcPpZIyXUGjJKJcM2xqrEodyiKOnIy4Xvrg";

let client;

function connectX() {
    setState(null);
    client = new EventSource("https://ditto-drogue-dev.apps.wonderful.iot-playground.org/api/2/things");
    client.onmessage = function(event) {
    };
    client.onopen = function() {
        state.connected = true;
    }
    client.onerror = function (msg) {
        console.log("Error", msg);
    }
}

function connect() {
    setState(null);
    client = new WebSocket("wss://ditto-drogue-dev.apps.wonderful.iot-playground.org/ws/2?access_token=" + encodeURIComponent(accessToken));
    client.addEventListener('open', function(event) {
        console.log("Connected: ", event);
        client.send("START-SEND-EVENTS");
    });
    client.addEventListener('message', function(event) {
        // console.log("Message: ", event);
        if (event.type !== "message") {
            return;
        }
        if (event.data === "START-SEND-EVENTS:ACK") {
            state.connected = true;
            return;
        }
        try {
            const data = JSON.parse(event.data);
            // console.log(data);
            const accel = data.value.features.accelerometer.properties;
            setState(accel);
        }
        catch(e) {}

    });
    client.addEventListener('error', function(event) {
        state.connected = false;
        console.log("Error: ", event);
    });
    client.addEventListener('close', function(event) {
        state.connected = false;
        console.log("Closed: ", event);
    });
}
