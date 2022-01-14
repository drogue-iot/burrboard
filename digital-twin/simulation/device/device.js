
const application = "ctron-test-ditto";
const device = "device1";
const password = "foobar";

let client;
let canSend = true;
let paused = false;
let lastSend = Date.now();

const SEND_DELAY = 1000 / 30; // ms, 30Hz

function setState(x,y,z) {
    setField('accel-x', x);
    setField('accel-y', y);
    setField('accel-z', z);

    const now = Date.now();

    if (client && canSend && !paused && ((now - lastSend > SEND_DELAY))) {
        canSend = false;
        lastSend = now;
        client.send("state", JSON.stringify({
            "value": {
                "features": {
                    "accelerometer": {
                        "properties": {
                            "x": x, "y": y, "z": z
                        }
                    }
                }
            }
        }), 0, false);
    }
}

function setField(id, value) {
    let field = document.getElementById(id);
    field.innerText = typeof value == 'number' && value.toFixed(1) || value;
}

function onConnect() {
    canSend = true;
    setField("mqtt", "Connected");
}

function toHex(dec) {
    return dec.toString(16).padStart(2, "0")
}

function randomClientId() {
    const id = new Uint8Array(20 / 2);
    window.crypto.getRandomValues(id)
    return Array.from(id, toHex).join('')
}

function init() {
    if (window.DeviceOrientationEvent) {
        setState("?", "?", "?");

        window.addEventListener("deviceorientation", function (event) {
            // alpha: rotation around z-axis
            const rotateDegrees = event.alpha;
            // gamma: left to right
            const leftToRight = event.gamma;
            // beta: front back motion
            const frontToBack = event.beta;

            setState(frontToBack, leftToRight, rotateDegrees);
        }, true);

        client = new Paho.Client("wss://mqtt-endpoint-ws-browser-drogue-dev.apps.wonderful.iot-playground.org/mqtt", randomClientId());
        client.onConnectionLost = function(responseObject) {
            if (responseObject.errorCode !== 0) {
                console.log("onConnectionLost: " + responseObject.errorMessage);
            }
            setField("mqtt", "Disconnected");
        };
        client.onMessageDelivered = function () {
            canSend = true;
        }

        setField("mqtt", "Connecting");

        client.connect({
            userName: device + "@" + application,
            password: password,
            cleanSession: true,
            reconnect: true,
            useSSL: true,
            mqttVersion: 4,
            onSuccess: onConnect,
            onFailure: function(err) {
                setField("mqtt", "Failed: " + JSON.stringify(err));
            }
        });

    } else {
        setState("n/a", "n/a", "n/a");
    }
}

function pause() {
    paused = true;
}

function resume() {
    paused = false;
}
