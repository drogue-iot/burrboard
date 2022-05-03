# Drogue BLE Mesh Gateway

## To run with burrboard

Follow [these](docs/burrboard.md) instructions.

## Getting started

* Create Drogue cloud application, gateway and device

```
drg create app ble-demo
drg create device --app ble-demo gateway
drg set password --app ble-demo gateway hey-rodney
drg create device --app ble-demo 00aa
drg set gateway --app ble-demo 00aa gateway
```

* Initialize two devices using [this procedure](docs/mesh.md).

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
TOKEN=159d79164ebff7f1 DROGUE_APPLICATION=app DROGUE_DEVICE=device1 DROGUE_PASSWORD=pass app/gateway.py
```
You should use proper Drogue cloud device device and password. The default values are set according to the example above (e.g. `gateway@ble-demo`)

* And the device

```
TOKEN=62cb5d464413e5c7 app/device.py
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

To run the gateway on a NUC installed with F35, [this](docs/nuc.md) should
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

Now you can run the gateway mounting a provisioned mesh configuration
for the `meshd` process within. We're using the host's dbus socket, so
we want to make sure the host's `meshd` process isn't running. If you
wanted to use the host's `meshd`, you could set `MESH=false` and omit
the `/var/lib/bluetooth/mesh` mount, as we do for the device simulator
below.

```
sudo docker run -it --rm \
--net=host --privileged --name drogue-gateway \
-v $PWD/example/mesh/:/var/lib/bluetooth/mesh/ \
-v /var/run/dbus/system_bus_socket:/var/run/dbus/system_bus_socket \
--env TOKEN=22daac25e4e87e69 \
docker.io/jcrossley3/drogue-gateway app/gateway.py
```

Note that we provided an example mesh state with some pre-joined
devices and known tokens we pass as environment variables. You can
provide your own mesh state similarly. And you can pass env vars for
DROGUE_APPLICATION, DROGUE_DEVICE, and DROGUE_PASSWORD as appropriate.

You can run a device simulator to send events every few seconds to the
gateway. Because we mount the shared dbus socket, we set `MESH=false`
to ensure we're talking to the correct `meshd` -- the one running in
the gateway container.

```
sudo docker run -it --rm \
--net=host --privileged --name drogue-device \
-v /var/run/dbus/system_bus_socket:/var/run/dbus/system_bus_socket \
--env MESH=false \
--env TOKEN=dcb67c7829fa7fa7 \
docker.io/jcrossley3/drogue-gateway app/device.py
```

## Run using Kubernetes

### Architecture

![containerized ble-mesh](https://user-images.githubusercontent.com/141611/166448688-8d7c59d7-3d63-484e-8cca-69409606de0c.svg)

### Deploy

You can run the gateway and the simulator in Kubernetes as well. We
tested it on [microshift](https://microshift.io/). Take a look at how
to install it on a NUC [here](docs/nuc.md#Microshift).

Once Kubernetes is installed and available, run the example to deploy
both a gateway and a simulated device that will report its status
every few seconds:

```
oc  new-project drogue
oc kustomize example/ | envsubst | oc apply -f -
```

We use [kustomize](https://kustomize.io/) to populate the image and
[envsubst](https://command-not-found.com/envsubst) to resolve the
`meshd` config path in the [deployment specs](example/deployment.yaml/).

Once the pods are `Running`, verify the gateway is sending the device
status to the cloud like so:

```
oc logs -f -l role=gateway
```

Similarly...

```
oc logs -f -l role=device
```

Shut down the example like so:

```
oc delete -k example/
```

## Run using Kubernetes and host bluetooth mesh

It's possible to use bluetooth mesh running on the host machine and run on applications containerized. We can do that by providing a system dbus socket to the pod.

There are two examples provided for this type of deployment. One is running just a [gateway](deploy/bluez/gateway-only/) and another with a few more devices already [provisioned](deploy/bluez/example/)

To deploy these workloads you first need to start a bluetooth-mesh deamon with appropriate configuration on the host, like

```
cd burrboard/gateway/
sudo /usr/libexec/bluetooth/bluetooth-meshd --config ${PWD}/deploy/bluez/example/meshcfg --storage ${PWD}/deploy/bluez/example/mesh --debug -nd
```

Then you can deploy the gateway

```
oc apply -f deploy/k8s/dbus-gateway.yaml
```

and optionaly device simulator

```
oc apply -f deploy/k8s/dbus-device.yaml
```

Make sure that pods are running and check the logs

```
oc project drogue
oc get pods
oc logs --tail=100 -f -l app=drogue
```

To run only gateway without any other devices, use the following path `${PWD}/deploy/bluez/gateway-only/meshcfg` for the mesh daemon and change appropriate token to `9f85d8ba0452b81c` in the k8s deployment.
