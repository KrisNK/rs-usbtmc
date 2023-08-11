# Rust USBTMC

Pure Rust implementation of the USBTMC protocol to connect to instruments.

Thus far, this library implements the basic USBTMC control endpoint commands, writing DEVICE_DEPENDENT messages to the BULK OUT endpoint and reading DEVICE_DEPENDENT messages to the BULK IN endpoint.

## Usage

To use, add the following line to your project's Cargo.toml dependencies:
```toml
rs-usbtmc = "0.1"
```


## Example

The example below demonstrates how to connect to, send commands to and query the device. 

```rust
use rs_usbtmc::UsbtmcClient;

const DEVICE_VID: u16 = 0x0000;
const DEVICE_PID: u16 = 0x0000;

fn main() {
    // connect to the device
    let device = UsbtmcClient::connect(DEVICE_VID, DEVICE_PID).expect("failed to connect");

    // send a command to the device
    device.command("*IDN?").expect("failed to send command");

    // query the device and get a string
    let response: String = device.query("*IDN?").expect("failed to query device");

    // query the device and get a bytes
    let response: Vec<u8> = device.query_raw("*IDN?").expect("failed to query device");
}
```

## Project Plans

I created this driver as part of a project to control an oscilloscope during a summer research position. Alone, I do not have access to an oscilloscope. If I do obtain one, the plan is to:

- Fully implement all possible requests
- Implement the usb488 subclass requests

I'll reach out to my university for access to an instrument to complete this project, but I'm open to collaborating.