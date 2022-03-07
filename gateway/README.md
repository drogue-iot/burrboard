# Drogue BLE Mesh Gateway

## Getting started

* Create Drogue cloud application, gateway and device

```
drg create app ble-demo
drg create device --app ble-demo gateway
drg set password --app ble-demo gateway hey-rodney
drg create device --app ble-demo 00aa
drg set gateway --app ble-demo 00aa gateway
```

* Initialize two devices using [this procedure](mesh.md).

Generate pub/sub connection between them. Let's say the first device have address `00aa` and the other one `00ae`, `00aa` will simulate a device and `00ae` a gateway.

* Create a pub setting for a device

```
pub-set 00aa C002 0 50 5 1000
pub-set 00aa C002 0 50 5 1100
```

* Create a subscription for a gateway.

```
sub-add 00ae c002 1000
sub-add 00ae c002 1100

```

* Now you can start gateway

```
TOKEN=159d79164ebff7f1 DROGUE_DEVICE=device1@app DROGUE_PASSWORD=pass ./gateway.py
```
You should use proper Drogue cloud device device and password. The default values are set according to the example above (e.g. `gateway@ble-demo`)

* And the device

```
TOKEN=62cb5d464413e5c7 ./device.py
```
in different terminals (or different machines)

* The device should be emitting status every 15 secs and gateway should forward that to the cloud.

You should be able to see data in the cloud by running

```
websocat wss://ws-integration.sandbox.drogue.cloud/ble-demo -H="Authorization: Bearer $(drg whoami -t)" | jq '.data_base64 |= @base64d'
```

* You can change the state of the device by sending a command to it, like

```
echo '{"state":"ON"}' | http POST https://api.sandbox.drogue.cloud/api/command/v1alpha1/apps/ble-demo/devices/00aa command==set-state "Authorization:Bearer $(drg whoami -t)"
```
