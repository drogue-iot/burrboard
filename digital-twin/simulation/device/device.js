'use strict';

const SEND_DELAY = 1000 / 30; // ms, 30Hz
const LEDS = ["1", "2", "3", "4"];

function setField(id, value) {
    let field = document.getElementById(id);
    field.innerText = typeof value == 'number' && value.toFixed(1) || value;
}

function toHex(dec) {
    return dec.toString(16).padStart(2, "0")
}

function randomClientId() {
    const id = new Uint8Array(20 / 2);
    window.crypto.getRandomValues(id)
    return Array.from(id, toHex).join('')
}

class Device {

    constructor(onLedChange) {
        this.lastSend = new Date();
        this.paused = false;
        this.counters = {
            "A": 0,
            "B": 0,
        };
        this.onLedChange = onLedChange;
        this.leds = Object.fromEntries(LEDS.map(led => [led, false]));
        this.temperature = null;
        this.light = null;

        this.setConnectionState("Disconnected");

        this.client = new Paho.Client("wss://mqtt-endpoint-ws-browser-drogue-dev.apps.wonderful.iot-playground.org/mqtt", randomClientId());
        this.client.onConnectionLost = (responseObject) => {
            if (responseObject.errorCode !== 0) {
                console.log("onConnectionLost: " + responseObject.errorMessage);
            }
            this.setConnectionState("Disconnected");
        };
        if (onLedChange) {
            this.client.onMessageArrived = (msg) => {
                this.commandInbox(msg);
            };
        }

        if (window.DeviceOrientationEvent) {
            this.showAccelState("?", "?", "?");
            window.addEventListener("deviceorientation", (event) => {
                // alpha: rotation around z-axis
                const rotateDegrees = event.alpha;
                // gamma: left to right
                const leftToRight = event.gamma;
                // beta: front back motion
                const frontToBack = event.beta;

                this.setAccelState({x: frontToBack, y: leftToRight, z: rotateDegrees});
            }, true);
        } else {
            this.showAccelState("n/a", "n/a", "n/a");
        }
    }

    setAccelState(state) {
        this.sendAccelUpdate({
            "x": state.x, "y": state.y, "z": state.z
        });
        this.showAccelState(state.x, state.y, state.z);
    }

    showAccelState(x, y, z) {
        setField('accel-x', x);
        setField('accel-y', y);
        setField('accel-z', z);
    }

    setConnectionState(connectionState) {
        this.connectionState = connectionState;
        setField("connection", connectionState);
    }

    connect(application, device, password) {
        if (this.client.isConnected()) {
            return;
        }

        this.setConnectionState("Connecting");

        this.client.connect({
            userName: device + "@" + application,
            password: password,
            cleanSession: true,
            reconnect: true,
            useSSL: true,
            mqttVersion: 4,
            onSuccess: () => {
                this.setConnectionState("Subscribing");
                this.client.subscribe("command/inbox/#", {
                    qos: 0,
                    timeout: 5,
                    onSuccess: () => {
                        this.connectionEstablished();
                        this.setConnectionState("Connected");
                    },
                    onFailure: (err) => {
                        this.setConnectionState("Subscribe failed: " + JSON.stringify(err));
                    }
                })
            },
            onFailure: (err) => {
                this.setConnectionState("Failed: " + JSON.stringify(err))
            }
        });
    }

    disconnect() {
        this.client.disconnect();
    }

    connectionEstablished() {
        this.sendLedUpdate();
        if (this.temperature !== null) {
            this.setTemperature(this.temperature);
        }
        if (this.light !== null) {
            this.setLight(this.light);
        }
    }

    sendAccelUpdate(data) {
        const now = Date.now();

        if (this.client.isConnected() && !this.paused && ((now - this.lastSend > SEND_DELAY))) {
            this.lastSend = now;
            this.updateFeature("accelerometer", data);
        }
    }

    sendLedUpdate() {
        this.updateFeature("leds", this.leds);
    }

    updateFeature(feature, properties) {
        if (!this.client.isConnected()) {
            return;
        }

        console.log(feature, " = ", properties);

        this.client.send("state", JSON.stringify({
            "path": "/features/" + encodeURIComponent(feature) + "/properties",
            "value": properties
        }), 0, false);
    }

    pause() {
        this.paused = true;
    }

    resume() {
        this.paused = false;
    }

    press(button) {
        this.counters[button] += 1;
        this.updateFeature("buttons", this.counters);
    }

    commandInbox(msg) {
        console.log("Command: ", msg);
        const segments = msg.topic.split("/", 4);
        console.log("Segments: ", segments);
        if (segments.length !== 4) {
            return;
        }

        try {
            this.handleCommand(segments[3], JSON.parse(msg.payloadString));
        } catch (err) {
            console.log("Failed to decode payload: ", err);
        }
    }

    handleCommand(command, payload) {
        console.log("Command: ", command, " Payload: ", payload);
        if (command === "leds") {
            this.leds = Object.assign(
                this.leds,
                payload
            );
            for (const ledsKey in this.leds) {
                if (LEDS.indexOf(ledsKey) === -1) {
                    delete this.leds[ledsKey];
                }
            }
            console.log("New LED state: ", this.leds);
            this.onLedChange(this.leds);
        }
    }

    setTemperature(value) {
        this.temperature = value;
        this.updateFeature("temperature", {"value": value});
    }

    setLight(value) {
        this.light = value;
        this.updateFeature("light", {"value": value});
    }

}
