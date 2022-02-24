'use strict';

const SEND_DELAY = 1000 / 30; // ms, 30Hz

// const ENDPOINT = "wss://mqtt-endpoint-ws-browser-drogue-dev.apps.wonderful.iot-playground.org/mqtt";
const ENDPOINT = "${ENDPOINT_URL}"; // replaced by the build process

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
        this.leds = [false, false, false, false];
        this.temperature = null;
        this.light = null;

        this.setConnectionState("Disconnected");

        this.client = new Paho.Client(ENDPOINT, randomClientId());
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
        this.sendAllLedsUpdate();
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

    sendLedUpdate(led) {
        this.updateFeature("led_" + led, this.leds[led-1]);
    }

    sendAllLedsUpdate() {
        this.updateFeature("led_1", {state: this.leds[0]});
        this.updateFeature("led_2", {state: this.leds[1]});
        this.updateFeature("led_3", {state: this.leds[2]});
        this.updateFeature("led_4", {state: this.leds[3]});
    }

    updateFeature(feature, properties) {
        if (!this.client.isConnected()) {
            return;
        }

        let features = {};
        features[feature] = properties;
        let payload = {features};

        console.log("Feature: ", feature, " properties: ", properties, " payload: ", payload);

        this.client.send("state", JSON.stringify(payload), 0, false);
    }

    pause() {
        this.paused = true;
    }

    resume() {
        this.paused = false;
    }

    press(button) {
        let presses = (this.counters[button] += 1);
        this.updateFeature("button_" + button.toLowerCase(), {presses});
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
        if (typeof command === "string" && command.startsWith("led_")) {

            let led = parseInt(command.substring(4 /* leds_ */), 10);
            if (!isNaN(led) && led > 0 && led <= this.leds.size ) {
                this.leds[led-1] = payload["state"] === true;
            }

            console.log("New LED state: ", this.leds);
            this.onLedChange(this.leds);
            this.sendLedUpdate(led);
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
