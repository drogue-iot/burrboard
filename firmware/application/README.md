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

```
cargo flash --manifest-path ../bootloader/Cargo.toml --release --chip nRF52840_xxAA
```

## Running

To run with debugger and enabling GATT service:

```
DEFMT_LOG=info cargo run --release --features defmt,gatt
```

To run with debugger and enabling MESH service:

```
DEFMT_LOG=info cargo run --release --features defmt,mesh
```
