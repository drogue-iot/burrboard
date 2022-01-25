```yaml
metadata:
  name: my-app
spec:
  ditto:
    exporter:
      kafka:
        bootstrap_servers: "kafka-kafka-bootstrap-burrboard.apps.wonderful.iot-playground.org:443"
        properties:
          sasl.mechanism: SCRAM-SHA-512
          sasl.username: user
          sasl.password: password
          security.protocol: SASL_SSL
        topic: sensor-events
      topics:
        - twinEvents:
            extraFields:
              - features/accelerometer
              - features/buttons
              - features/leds
              - features/light
              - features/temperature
              - features/state
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
