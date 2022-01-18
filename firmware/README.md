# burrboard firmware

This is the firmware for the burrboard application. 


## Prerequisites

* [uf2 utils]
* [cargo-binutils](https://github.com/rust-embedded/cargo-binutils)

### Uf2 utils

* Clone the [uf2 utils] git repository.
* Add the utils/ folder of your local copy to your $PATH environment variable.
 
 ### cargo-binutils
 
 ```
 cargo install cargo-binutils
 rustup component add llvm-tools-preview
 ```

## Running

* Make sure the feather is in bootloader mode (double-press the reset button)
* Set the MEDIA env to match your system

```
MEDIA=/run/media/lulf/FTHR840BOOT cargo run --release
```

NOTE: To watch debug output, connect a serial console to the UART TX/RX pins.
