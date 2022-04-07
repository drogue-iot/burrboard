# Drogue BLE Mesh Gateway

## To run with burrboard

Follow [these](burrboard.md) instructions.

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

Generate pub/sub connection between them. Let's say the first device have address `00aa` and the other one `00ac`, `00aa` will simulate a device and `00ac` a gateway.

* Create a pub setting for a device

```
pub-set 00aa C002 0 50 5 1000
pub-set 00aa C002 0 50 5 1100
```

* Create a subscription for a gateway.

```
sub-add 00ac c002 1000
sub-add 00ac c002 1100

```

* Now you can start gateway

```
TOKEN=159d79164ebff7f1 DROGUE_APPLICATION=app DROGUE_DEVICE=device1 DROGUE_PASSWORD=pass ./gateway.py
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

## To run gateway on NUC running Fedora

To run the gateway on a NUC installed with F35, [this](nuc.md) should
get you pretty close. It's handy to have `test-mesh` around -- from
the bluez source tree -- on which all the python scripts are based...

## Run using containers

You can containerize the gateway by running

```
docker build -t drogue-gateway .
```

or if you're building for a different platform

```
 docker buildx build --platform linux/arm -t drogue-gateway .
 ```

 You can then tag/push containers in a regular fashion, e.g.

 ```
 docker tag drogue-gateway quay.io/dejanb/drogue-gateway
 docker push quay.io/dejanb/drogue-gateway
 ```

 Now you can run gateway

 ```
 sudo docker run -it \
--net=host --privileged --name drogue-gateway \
-v $PWD/deploy/bluez/config_db.json:/root/.config/meshcfg/config_db.json \
-v $PWD/deploy/bluez/mesh/:/var/lib/bluetooth/mesh/ \
--env TOKEN=a4d2100e2fc8f6e5 \
quay.io/dejanb/drogue-gateway:latest app/gateway.py
```

or device simulator

```
sudo docker run -it \
--net=host --privileged \
--name drogue-gateway \
-v $PWD/deploy/bluez/config_db.json:/root/.config/meshcfg/config_db.json \
-v $PWD/deploy/bluez/mesh/:/var/lib/bluetooth/mesh/ \
--env TOKEN=bf2aadd0a6b0da55 \
quay.io/dejanb/drogue-gateway app/device.py
```

Note that we provided example mesh state with some pre-joined devices. You can provide your own mesh state similarly.

## Run using Kubernetes

You can run the gateway and the simulator in Kubernetes as well. We tested it on Microshift for now. Take a look at how set
Microshift on NUC, [here](nuc.md#Microshift).

Once, you have your K8s ready run

```
oc  new-project drogue
oc apply -f deploy/k8s/gateway.yaml
```

or

```
oc  new-project drogue
oc apply -f deploy/k8s/device.yaml
```

Make sure you adjust local paths in yaml files. This will be improved soon.