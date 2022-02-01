# burrboard firmware

This is the firmware for the burrboard application.  This works with both the nRF feather sense, the
nRF feather express, and the nRF52840 DK.

## Prerequisites

* `probe-run`
* `probe-rs-cli`
* `cargo-flash`


## Installing softdevice

Download `softdevice.hex` from https://github.com/drogue-iot/burrboard/releases/tag/firmware-0.1

```
probe-rs-cli download softdevice.hex --format Hex --chip nRF52840_xxAA
```

## Installing bootloader

Download `bootloader.hex` from https://github.com/drogue-iot/burrboard/releases/tag/firmware-0.1

```
probe-rs-cli download bootloader.hex --base-address 0xfc000 --format Hex --chip nRF52840_xxAA
```

## Running

To run with debugger and enabling GATT service:

```
DEFMT_LOG=info cargo run --release --features defmt,gatt
```
