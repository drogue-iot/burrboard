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
        this.sendTimer = null;
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
        this.accel = null;

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
        this.sendState();
    }

    sendAccelUpdate(data) {
        this.accel = data;
        this.sendState();
    }

    // Send the current state
    sendState() {

        let payload = {
            features: {
                temperature: {
                    value: this.temperature
                },
                light: {
                    value: this.light
                },
                button_a: {
                    presses: this.counters.A,
                },
                button_b: {
                    presses: this.counters.B,
                },
                led_1: {
                    state: this.leds[0],
                },
                led_2: {
                    state: this.leds[1],
                },
                led_3: {
                    state: this.leds[2],
                },
                led_4: {
                    state: this.leds[3],
                },
                accelerometer: self.accel,
            }
        };

        const now = Date.now();
        if (this.client.isConnected() && !this.paused) {
            if (now - this.lastSend > SEND_DELAY) {

                // if we have a send timer pending
                if (this.sendTimer) {
                    // clear timeout, reset
                    window.clearTimeout(this.sendTimer);
                    this.sendTimer = null;
                }

                this.sendTimer = null;
                this.lastSend = now;
                const json = JSON.stringify(payload);
                console.log("Sending state: ", payload, " JSON: ", json);
                this.client.send("state", json, 0, false);

            } else {
                // too soon ...
                if (!this.sendTimer) {
                    // ... schedule sending, as we currently have no send pending
                    this.sendTimer = window.setTimeout(() => {
                        this.sendState();
                    });
                }
            }
        }

    }

    pause() {
        this.paused = true;
    }

    resume() {
        this.paused = false;
        this.sendState();
    }

    press(button) {
        this.counters[button] += 1;
        this.sendState();
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
            if (!isNaN(led) && led > 0 && led <= this.leds.size) {
                this.leds[led - 1] = payload["state"] === true;
            }

            console.log("New LED state: ", this.leds);
            this.onLedChange(this.leds);
            this.sendState();
        }
    }

    setTemperature(value) {
        this.temperature = value;
        this.sendState();
    }

    setLight(value) {
        this.light = value;
        this.sendState();
    }

}
