
function onConnect() {
    // Once a connection has been made, make a subscription and send a message.
    console.log("onConnect");
    setField("mqtt", "Subscribing");
    client.subscribe("app/ctron-test-ditto", {
        onSuccess: function () { setField("mqtt", "Subscribed"); state.connected = true; },
        onFailure: function (err) { setField ( "mqtt", "Subscription failed: " + JSON.stringify(err))}
    })
}

function onFailure(err) {
    setField("mqtt", "Failed: " + JSON.stringify(err));
    state.connected = false;
}

function toHex(dec) {
    return dec.toString(16).padStart(2, "0")
}

function randomClientId() {
    const id = new Uint8Array(20 / 2);
    window.crypto.getRandomValues(id)
    return Array.from(id, toHex).join('')
}

function connect() {
    setState(null);

    client = new Paho.Client("wss://mqtt-integration-ws-browser-drogue-dev.apps.wonderful.iot-playground.org/mqtt", randomClientId());

    client.onConnectionLost = function(responseObject) {
        if (responseObject.errorCode !== 0) {
            console.log("onConnectionLost: " + responseObject.errorMessage);
        }
        setField("mqtt", "Disconnected");
    };
    client.onMessageArrived = function(msg) {
        let data = JSON.parse(msg.payloadString).data;
        let accel = data.value.features.accelerometer.properties;
        setState(accel);
    };

    setField("mqtt", "Connecting");

    client.connect({
        cleanSession: true,
        reconnect: true,
        useSSL: true,
        mqttVersion: 4,
        onSuccess: onConnect,
        onFailure: onFailure
    });

}
