
```shell
USER="someuser"
PASSWORD="abc"
podman run --rm -ti docker.io/bitnami/kafka:latest kafka-console-consumer.sh \
	--topic sensor-events \
	--bootstrap-server kafka-kafka-bootstrap-burrboard.apps.wonderful.iot-playground.org:443 \
	--consumer-property security.protocol=SASL_SSL \
	--consumer-property sasl.mechanism=SCRAM-SHA-512 \
	--consumer-property sasl.jaas.config='org.apache.kafka.common.security.scram.ScramLoginModule required username="${USER}" password="${PASSWORD}";'
```

Also get the headers:

```shell
set TOPIC sensor-events
set USER "foo"
set PASSWORD (oc -n burrboard get secret $USER -o json | jq -r '.data.password' | base64 -d)
podman run --rm -it docker.io/edenhill/kcat:1.7.1 \
	-b kafka-kafka-bootstrap-burrboard.apps.wonderful.iot-playground.org:443 \
	-C \
	-t $TOPIC \
	-X security.protocol=SASL_SSL \
	-X sasl.mechanism=SCRAM-SHA-512 \
	-X sasl.username=$USER \
	-X sasl.password=$PASSWORD \
	-f '\nKey (%K bytes): %k
  Value (%S bytes): %s
  Timestamp: %T
  Partition: %p
  Offset: %o
  Headers: %h\n'
```

Get state:

```shell
set TOKEN (drg whoami -t)
http -v GET https://ditto-drogue-dev.apps.wonderful.iot-playground.org/api/2/things/ctron-test-ditto:device1 "Authorization:Bearer $TOKEN"
```

Request LED state:

```shell
set TOKEN (drg whoami -t)
http -v PUT https://ditto-drogue-dev.apps.wonderful.iot-playground.org/api/2/things/ctron-test-ditto:device1/features/leds "Authorization:Bearer $TOKEN" \
  --raw '{}'
http -v PUT https://ditto-drogue-dev.apps.wonderful.iot-playground.org/api/2/things/ctron-test-ditto:device1/features/leds/desiredProperties/1 "Authorization:Bearer $TOKEN" \
  --raw 'true'
```
