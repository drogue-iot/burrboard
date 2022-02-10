
# Cheatsheet

Setup without digital twin.

## Creating the application

```shell
drg create application burrboard
```

## Creating devices (for the simulator)

Requires drg >= 0.8.1 (alpha1)

```shell
drg create device -a burrboard device1
drg set password -a burrboard foobar1 device1
```

Create a list of devices:

```shell
for i in $(seq -w 1 10); do
  drg create device -a burrboard device$i
  drg set password -a burrboard foobar$i device$i
done
```

Delete them again:

```shell
for i in $(seq -w 1 10); do
  drg delete device -a burrboard device$i
done
```

## Access data stream

Also get the headers:

```shell
set TOPIC "events-burrboard"
set USER "user-burrboard"
set PASSWORD "this-is-not-the-password"
podman run --rm -it docker.io/edenhill/kcat:1.7.1 \
         -b drogue-iot-kafka-bootstrap-drogue-dev.apps.wonderful.iot-playground.org:443 \
         -o end \
         -X security.protocol=SASL_SSL \
         -X sasl.mechanism=SCRAM-SHA-512 \
         -X sasl.username=$USER \
         -X sasl.password=$PASSWORD \
         -f '\nKey (%K bytes): %k
       Value (%S bytes): %s
       Timestamp: %T
       Partition: %p
       Offset: %o
       Headers: %h\n' \
         -G my-kafka-group $TOPIC
```

## Device commands

Request LED state:

```shell
set TOKEN (drg whoami -t)
echo '{"1":true}' | http POST https://api-drogue-dev.apps.wonderful.iot-playground.org/api/command/v1alpha1/apps/burrboard/devices/device01 command==leds "Authorization:Bearer $TOKEN"
```
