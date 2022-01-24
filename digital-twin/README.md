
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
set USER "foo"
set PASSWORD "bar"
podman run --rm -it docker.io/edenhill/kcat:1.7.1 \
	-b kafka-kafka-bootstrap-burrboard.apps.wonderful.iot-playground.org:443 \
	-C \
	-t sensor-events \
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
