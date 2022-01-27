```yaml
metadata:
  name: my-app
spec:
  ditto:
    exporters:
    - kafka:
        bootstrapServers: "kafka-kafka-bootstrap-burrboard.apps.wonderful.iot-playground.org:443"
        properties:
          sasl.mechanism: SCRAM-SHA-512
          sasl.username: user
          sasl.password: password
          security.protocol: SASL_SSL
      targets:
        - topic: ditto-events
          mode:
            ditto:
              normalized: true
          subscriptions:
            - twinEvents:
                - extraFields:
                    - features/state
                    - features/leds
        - topic: sensor-events
          mode:
            cloudEvents:
              normalized: true
          subscriptions:
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
