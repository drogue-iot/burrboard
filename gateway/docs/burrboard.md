# Setting up burrboard with gateway

Components:

* Burrboard (00ac)
* Provisioner (mesh-cfgclient)
* Gateway (00aa)

## Prepare provisioner

```
create
appkey-create 0 0
```

## Provision gateway

In terminal:

```
./device.py join
```

The device terminal should display a UUID that you then paste into the provisioner.

in provisioner (mesh-cfgclient)

```
# provision <UUID>
```

Once provisioned, the node's primary address, e.g. `00aa`, will be
displayed, and the device terminal will display the token you must
pass to the `gateway.py` script later, so make note of it.

## Configure gateway

Bind the gateway sensor model

```
# menu config
# target 00aa
# composition-get
# appkey-add 0
# bind 00aa 0 1100
# virt-add
# sub-add 00aa a112 1100
```

The `a112` is the virtual address returned by `virt-add`.

## Provision burrboard

In mesh-cfgclient:

```
# provision <UUID of burrboard>
```

We'll assume the primary address returned is `00ac`.

If you don't know the UUID of the board...

```
# discover-unprovisioned on
```

## Configure burrboard

In mesh-cfgclient:

```
# menu config
# target 00ac
# composition-get
# appkey-add 0
# bind 00ac 0 1100
# pub-set 00ac a112 0 50 5 1100
```

Note the `a112` address from the call to `virt-add` above.

## Create drogue cloud devices and start gateway

```
drg create app ble-demo
drg create device --app ble-demo gateway
drg set password --app ble-demo gateway hey-rodney
drg create device --app ble-demo 00ac
drg set gateway --app ble-demo 00ac gateway
TOKEN=<token from device terminal> DROGUE_APPLICATION=ble-demo DROGUE_DEVICE=gateway DROGUE_PASSWORD=hey-rodney app/gateway.py
```
