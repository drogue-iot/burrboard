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


in provisioner (mesh-cfgclient)

```
# provision <UUID>
```

The device terminal should display a token that you must paste into the provisioner.

Once provisioned, the device terminal will give you a token to use for the gateway process.

## Configure gateway

Bind the gateway sensor model

```
# menu config
# target 00aa
# composition-get
# appkey-add 0
# bind 00aa 0 1100
```

## Provision burrboard

In mesh-cfgclient:

```
# provision <UUID of burrboard>
```

## Configure burrboard

In mesh-cfgclient:

```
# menu config
# target 00ac
# composition-get
# appkey-add 0
# bind 00ac 0 1100
# pub-set 00ac 00aa 0 50 5 1100
```

## Create drogue cloud devices and start gateway

```
drg create app ble-demo
drg create device --app ble-demo gateway
drg set password --app ble-demo gateway hey-rodney
drg create device --app ble-demo 00ac
drg set gateway --app ble-demo 00ac gateway
TOKEN=<token from device terminal> DROGUE_APPLICATION=ble-demo DROGUE_DEVICE=gateway DROGUE_PASSWORD=hey-rodney ./gateway.py
```
