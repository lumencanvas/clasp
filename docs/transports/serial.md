---
title: Serial
description: UART/USB serial transport for microcontrollers and hardware devices
order: 6
---

# Serial Transport

Serial transport carries CLASP frames over UART/RS-232/USB-Serial connections. This is how microcontrollers (Arduino, ESP32, STM32) and other hardware devices speak CLASP.

## How Serial Devices Reach the Relay

Serial is a point-to-point connection between two devices -- it doesn't go over a network. To connect a serial device to the CLASP relay, you need an intermediate machine:

```
Arduino ──[USB/Serial]──> Raspberry Pi ──[WebSocket]──> clasp-relay
                          running CLASP                  :7330
                          client with
                          serial transport
```

The Raspberry Pi (or any computer) runs a CLASP client that:
1. Opens the serial port
2. Reads CLASP binary frames from the Arduino
3. Forwards them to the relay over WebSocket
4. Forwards relay responses back to the Arduino over serial

The Arduino sends real CLASP binary frames -- no translation happens. The intermediate machine is just a transparent hop between two transports.

## Feature Flag

Serial is not included in default features:

```bash
cargo build --features serial
```

## Connection

```
serial:///dev/ttyUSB0?baud=115200
serial:///dev/tty.usbserial-1420?baud=115200
serial://COM3?baud=115200
```

## Rust Transport API

```rust
use clasp_transport::serial::{SerialTransport, SerialConfig, SerialParity, SerialFlowControl};

let config = SerialConfig {
    port: "/dev/ttyUSB0".into(),
    baud_rate: 115200,
    data_bits: 8,
    stop_bits: 1,
    parity: SerialParity::None,
    flow_control: SerialFlowControl::None,
};

let (sender, receiver) = SerialTransport::new(config).await?;

// Same trait as every other transport
sender.send(clasp_frame).await?;
if let Some(TransportEvent::Data(bytes)) = receiver.recv().await {
    // handle frame
}
```

## Frame Delimiting

Serial is a raw byte stream with no message boundaries. The serial transport uses **SLIP (Serial Line Internet Protocol)** framing by default to delimit CLASP frames:

```
SLIP encoding:
  END  = 0xC0
  ESC  = 0xDB
  ESC_END = 0xDC  (escaped END byte)
  ESC_ESC = 0xDD  (escaped ESC byte)

Wire format: [CLASP frame bytes with escaping] 0xC0
```

This is handled automatically. You send and receive complete CLASP frames.

## Common Baud Rates

| Rate | Typical Use |
|------|-------------|
| 9600 | Legacy devices, long cable runs |
| 115200 | Common default for most microcontrollers |
| 250000 | DMX protocol (special case) |
| 921600 | High-speed USB-serial |
| 1000000+ | USB-serial adapters with fast MCUs |

## Embedded Examples

### Arduino / ESP32

```cpp
#include <CLASP.h>

CLASPSerial clasp(Serial);

void setup() {
    Serial.begin(115200);
    clasp.begin();
}

void loop() {
    clasp.set("/sensor/temp", readTemp());
    clasp.update();  // process incoming frames
    delay(1000);
}
```

### Rust (no_std)

```rust
use clasp_embedded::{Client, SerialFramer};

let mut client = Client::new();
let mut framer = SerialFramer::new();

// Send
let frame = client.prepare_set("/sensor/temp", Value::Float(23.5));
uart_send(framer.encode(&frame));

// Receive
while let Some(byte) = uart_recv() {
    if let Some(frame) = framer.decode(byte) {
        handle_message(frame);
    }
}
```

### Web Serial API (Browser)

Chrome supports serial ports directly:

```javascript
const port = await navigator.serial.requestPort()
await port.open({ baudRate: 115200 })

const transport = new SerialTransport(port)
const client = await new ClaspBuilder(transport)
await client.set('/sensor/value', 42)
```

## Finding Serial Ports

### Linux
```bash
ls /dev/ttyUSB* /dev/ttyACM*
# Permissions: sudo usermod -a -G dialout $USER
```

### macOS
```bash
ls /dev/tty.usb* /dev/cu.usb*
```

### Windows
Ports are `COM1`, `COM2`, etc. Check Device Manager.

## Troubleshooting

**No communication** -- Check port name, baud rate, and permissions. On Linux, your user needs to be in the `dialout` group. Verify TX/RX wiring isn't swapped.

**Garbled data** -- Baud rate mismatch between the two devices. Both sides must agree on baud rate, data bits, parity, and stop bits.

**Buffer overflow / lost frames** -- Enable hardware flow control (RTS/CTS) if your hardware supports it. Or reduce the message rate.

**Port busy** -- Another process has the serial port open. Only one process can use a serial port at a time.

## See Also

- [BLE Transport](ble.md) -- wireless alternative for microcontrollers
- [Core Concepts: Transports](../core/transports.md) -- how transports fit into CLASP
