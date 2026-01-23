# Frame Format

CLASP uses a minimal binary frame format optimized for parsing efficiency. This document describes the wire format for all CLASP frames.

## Frame Structure

```
┌─────────────────────────────────────────────────────────────────┐
│ Byte 0:     Magic (0x53 = 'S')                                  │
│ Byte 1:     Flags                                               │
│             [7:6] QoS (00=fire, 01=confirm, 10=commit, 11=rsv)  │
│             [5]   Timestamp present                             │
│             [4]   Encrypted                                     │
│             [3]   Compressed                                    │
│             [2:0] Encoding (000=msgpack, 001=binary)            │
│ Byte 2-3:   Payload Length (uint16 big-endian, max 65535)       │
├─────────────────────────────────────────────────────────────────┤
│ [If timestamp flag] Bytes 4-11: Timestamp (uint64 µs)           │
├─────────────────────────────────────────────────────────────────┤
│ Payload (binary encoding or MessagePack)                        │
└─────────────────────────────────────────────────────────────────┘
```

**Minimum frame size:** 4 bytes (header only, empty payload)
**Maximum frame size:** 65,539 bytes (4 header + 65,535 payload)
**With timestamp:** 12 bytes header + payload

## Header Fields

### Magic Byte (Byte 0)

Always `0x53` (ASCII 'S' for CLASP). Used for frame synchronization.

### Flags Byte (Byte 1)

```
Bit 7-6: QoS Level
  00 = Fire (no acknowledgment)
  01 = Confirm (acknowledgment required)
  10 = Commit (durable, acknowledgment required)
  11 = Reserved

Bit 5: Timestamp Present
  0 = No timestamp in header
  1 = 8-byte timestamp follows header

Bit 4: Encrypted
  0 = Payload is plaintext
  1 = Payload is encrypted

Bit 3: Compressed
  0 = Payload is uncompressed
  1 = Payload is LZ4 compressed

Bit 2-0: Encoding
  000 = MessagePack (legacy)
  001 = Binary (default)
  010-111 = Reserved
```

### Payload Length (Bytes 2-3)

16-bit unsigned integer, big-endian. Maximum value: 65,535 bytes.

### Timestamp (Bytes 4-11, optional)

64-bit unsigned integer, big-endian. Microseconds since session start or Unix epoch.

## Payload Format

### Binary Encoding (Default)

Messages use positional binary encoding for maximum efficiency.

#### SET Message (0x21)

```
┌──────────┬──────────┬────────────┬─────────┬──────────┬──────────┐
│ MsgType  │ Flags    │ AddrLen    │ Address │ Value    │ [Rev]    │
│ 0x21     │ vtype+fl │ u16        │ UTF-8   │ encoded  │ u64?     │
└──────────┴──────────┴────────────┴─────────┴──────────┴──────────┘
  1 byte    1 byte     2 bytes      variable  variable   0 or 8 bytes

Flags byte:
  [7]   has_revision - if set, 8-byte revision follows value
  [6]   lock - request exclusive lock
  [5]   unlock - release lock
  [3:0] value_type - type code of value
```

**Example:** SET `/test` to 0.75

```
Frame header:
53 01 00 1B        Magic='S', Flags=QoS.Confirm+Binary, Length=27

Payload:
21                 MsgType = SET (0x21)
07                 Flags = value_type f64 (0x07)
00 05              AddrLen = 5
2F 74 65 73 74     Address = "/test"
3F E8 00 00 00 00 00 00  Value = 0.75 (f64)

Total: 4 + 27 = 31 bytes
```

#### PUBLISH Message (0x20)

```
┌──────────┬──────────┬────────────┬─────────┬──────────┐
│ MsgType  │ Flags    │ AddrLen    │ Address │ Payload  │
│ 0x20     │ sigtype  │ u16        │ UTF-8   │ encoded  │
└──────────┴──────────┴────────────┴─────────┴──────────┘

Flags byte:
  [3:0] signal_type - 0=event, 1=stream, 2=gesture
```

#### SUBSCRIBE Message (0x10)

```
┌──────────┬──────────┬────────────┬─────────┬──────────┐
│ MsgType  │ SubId    │ PatternLen │ Pattern │ Options  │
│ 0x10     │ u16      │ u16        │ UTF-8   │ encoded  │
└──────────┴──────────┴────────────┴─────────┴──────────┘
```

#### HELLO Message (0x01)

```
┌──────────┬──────────┬────────────┬─────────┬──────────┐
│ MsgType  │ Version  │ NameLen    │ Name    │ Features │
│ 0x01     │ u8       │ u16        │ UTF-8   │ encoded  │
└──────────┴──────────┴────────────┴─────────┴──────────┘
```

#### BUNDLE Message (0x30)

```
┌──────────┬──────────┬────────────┬────────────────────┐
│ MsgType  │ Flags    │ MsgCount   │ Messages...        │
│ 0x30     │ flags    │ u16        │ length+payload×N   │
└──────────┴──────────┴────────────┴────────────────────┘

Flags byte:
  [7] has_timestamp - bundle is scheduled for future execution

Each embedded message:
┌────────────┬────────────────┐
│ MsgLen     │ Message        │
│ u16        │ (no frame hdr) │
└────────────┴────────────────┘
```

### MessagePack Encoding (Legacy)

When encoding bits are `000`, the payload is MessagePack-encoded:

```javascript
{
  type: "SET",
  address: "/test",
  value: 0.75
}
```

MessagePack is auto-detected by the first payload byte:
- `0x80-0x8F`: fixmap
- `0xDE`: map 16
- `0xDF`: map 32

## Size Comparison

For a typical SET message (`/test` = 0.75):

| Encoding | Size |
|----------|------|
| Binary CLASP | 31 bytes |
| MessagePack | 35 bytes |
| JSON | ~45 bytes |
| Verbose JSON | ~80 bytes |

Binary encoding is **55% smaller** than typical JSON.

## Fragmentation

CLASP frames have a maximum payload of 65,535 bytes. For larger messages:

1. **Split at application level** - Break large data into multiple addresses
2. **Use streaming** - For media, use a dedicated media protocol
3. **Compress** - Enable LZ4 compression for large payloads

## Encryption

When the encrypted flag is set:

```
┌────────────────┬────────────────────────────────────┐
│ Frame Header   │ Encrypted Payload                  │
│ (4-12 bytes)   │ (ChaCha20-Poly1305)                │
└────────────────┴────────────────────────────────────┘
```

The payload is encrypted using ChaCha20-Poly1305 with:
- 256-bit key (from capability token or session key)
- 96-bit nonce (incrementing per message)
- 128-bit authentication tag

## Compression

When the compressed flag is set:

```
┌────────────────┬──────────────┬────────────────────┐
│ Frame Header   │ Uncompressed │ LZ4 Compressed     │
│ (4-12 bytes)   │ Length (u32) │ Payload            │
└────────────────┴──────────────┴────────────────────┘
```

The uncompressed length is stored first (4 bytes, big-endian), followed by LZ4 block-compressed data.

## Transport Considerations

### WebSocket

- Use binary message type
- One CLASP frame per WebSocket message
- No additional framing needed

### UDP

- One CLASP frame per UDP datagram
- Maximum frame size should respect MTU (~1400 bytes)
- Consider using QUIC for larger messages

### TCP/Serial

- Frames are self-delimiting (magic + length)
- Scan for magic byte to resync after errors
