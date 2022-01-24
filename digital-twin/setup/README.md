```yaml
metadata:
  name: my-app
spec:
  ditto:
    exporter:
      kafka:
        bootstrap_servers: "kafka-kafka-bootstrap-burrboard.apps.wonderful.iot-playground.org:443"
        properties:
          sasl.jaas.config: "org.apache.kafka.common.security.scram.ScramLoginModule required username=\"abc\" password=\"xyz\";"
          sasl.mechanism: SCRAM-SHA-512
          security.protocol: SASL_SSL
        topic: sensor-events
      topics:
        - twinEvents:
            extraFields:
              - features/accelerometer
  publish:
    rules:
      - then:
          - setAttribute:
              name: dataschema
              value: "urn:eclipse:ditto"
          - setAttribute:
              name: subject
              value: modify
        when:
          isChannel: state
```
