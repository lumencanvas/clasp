# Data Types

CLASP uses a compact binary encoding for efficient transmission. This document describes all supported data types and their wire format.

## Scalar Types

| Type | Code | Size | Description |
|------|------|------|-------------|
| `null` | 0x00 | 1 byte | No value |
| `false` | 0x01 | 1 byte | Boolean false |
| `true` | 0x02 | 1 byte | Boolean true |
| `i32` | 0x05 | 5 bytes | Signed 32-bit integer |
| `i64` | 0x06 | 9 bytes | Signed 64-bit integer |
| `f64` | 0x07 | 9 bytes | 64-bit float (IEEE 754) |
| `str` | 0x08 | 3+ bytes | Length-prefixed UTF-8 string |
| `bin` | 0x09 | 5+ bytes | Length-prefixed binary data |
| `array` | 0x0A | 5+ bytes | Length-prefixed array |
| `map` | 0x0B | 5+ bytes | Length-prefixed map |

## Encoding Details

### Null (0x00)

```
┌────────┐
│  0x00  │
└────────┘
  1 byte
```

### Boolean (0x01, 0x02)

```
False:        True:
┌────────┐    ┌────────┐
│  0x01  │    │  0x02  │
└────────┘    └────────┘
  1 byte        1 byte
```

### Integer (0x05, 0x06)

32-bit and 64-bit signed integers, big-endian:

```
i32 (value: 42):
┌────────┬────────┬────────┬────────┬────────┐
│  0x05  │  0x00  │  0x00  │  0x00  │  0x2A  │
└────────┴────────┴────────┴────────┴────────┘
  type     ────────── value (big-endian) ──────

i64 (value: 42):
┌────────┬────────┬────────┬────────┬────────┬────────┬────────┬────────┬────────┐
│  0x06  │  0x00  │  0x00  │  0x00  │  0x00  │  0x00  │  0x00  │  0x00  │  0x2A  │
└────────┴────────┴────────┴────────┴────────┴────────┴────────┴────────┴────────┘
```

### Float (0x07)

64-bit IEEE 754 double-precision, big-endian:

```
f64 (value: 0.75):
┌────────┬────────────────────────────────────────────────────────────────────┐
│  0x07  │  0x3F  0xE8  0x00  0x00  0x00  0x00  0x00  0x00                    │
└────────┴────────────────────────────────────────────────────────────────────┘
  type     ────────────────── value (IEEE 754 big-endian) ──────────────────
```

### String (0x08)

Length-prefixed UTF-8 string. Length is u16 big-endian:

```
str (value: "hello"):
┌────────┬────────┬────────┬────────┬────────┬────────┬────────┬────────┐
│  0x08  │  0x00  │  0x05  │  0x68  │  0x65  │  0x6C  │  0x6C  │  0x6F  │
└────────┴────────┴────────┴────────┴────────┴────────┴────────┴────────┘
  type     ─ length ─       ──────────── UTF-8 bytes ────────────
           (5)              h      e      l      l      o
```

Maximum string length: 65,535 bytes.

### Binary (0x09)

Length-prefixed binary data. Length is u32 big-endian:

```
bin (value: [0xDE, 0xAD, 0xBE, 0xEF]):
┌────────┬────────┬────────┬────────┬────────┬────────┬────────┬────────┬────────┐
│  0x09  │  0x00  │  0x00  │  0x00  │  0x04  │  0xDE  │  0xAD  │  0xBE  │  0xEF  │
└────────┴────────┴────────┴────────┴────────┴────────┴────────┴────────┴────────┘
  type     ────────── length (u32) ──────────  ──────── data ────────
```

Maximum binary length: 4,294,967,295 bytes.

### Array (0x0A)

Length-prefixed array of typed values. Count is u32 big-endian:

```
array (value: [1, 2, 3]):
┌────────┬────────┬────────┬────────┬────────┬─────────────────────────────────┐
│  0x0A  │  0x00  │  0x00  │  0x00  │  0x03  │  [i32:1]  [i32:2]  [i32:3]      │
└────────┴────────┴────────┴────────┴────────┴─────────────────────────────────┘
  type     ────────── count (u32) ──────────   ──────── elements ────────
```

### Map (0x0B)

Length-prefixed map of string keys to typed values. Count is u32 big-endian:

```
map (value: {"x": 1, "y": 2}):
┌────────┬────────┬────────┬────────┬────────┬─────────────────────────────────┐
│  0x0B  │  0x00  │  0x00  │  0x00  │  0x02  │  [str:"x"][i32:1][str:"y"][i32:2]│
└────────┴────────┴────────┴────────┴────────┴─────────────────────────────────┘
  type     ────────── count (u32) ──────────   ──────── key-value pairs ───────
```

## Extension Types (Creative Primitives)

For creative applications, CLASP defines extension types for common data structures:

| Type | Code | Size | Description |
|------|------|------|-------------|
| `vec2` | 0x10 | 9 bytes | 2D vector (f32×2) |
| `vec3` | 0x11 | 13 bytes | 3D vector (f32×3) |
| `vec4` | 0x12 | 17 bytes | 4D vector (f32×4) |
| `color` | 0x13 | 5 bytes | RGBA color (u8×4) |
| `colorf` | 0x14 | 17 bytes | RGBA float (f32×4) |
| `mat3` | 0x15 | 37 bytes | 3×3 matrix (f32×9) |
| `mat4` | 0x16 | 65 bytes | 4×4 matrix (f32×16) |

### Vector Types

```
vec2 (value: [0.5, 0.3]):
┌────────┬────────────────────────┬────────────────────────┐
│  0x10  │  0x3F 0x00 0x00 0x00   │  0x3E 0x99 0x99 0x9A   │
└────────┴────────────────────────┴────────────────────────┘
  type     ──── x (f32) ────        ──── y (f32) ────

vec3 (value: [x, y, z]):
┌────────┬──── x (f32) ────┬──── y (f32) ────┬──── z (f32) ────┐
│  0x11  │                 │                 │                 │
└────────┴─────────────────┴─────────────────┴─────────────────┘

vec4 (value: [x, y, z, w]):
┌────────┬──── x ────┬──── y ────┬──── z ────┬──── w ────┐
│  0x12  │           │           │           │           │
└────────┴───────────┴───────────┴───────────┴───────────┘
```

### Color Types

```
color (value: RGBA 255, 128, 64, 255):
┌────────┬────────┬────────┬────────┬────────┐
│  0x13  │  0xFF  │  0x80  │  0x40  │  0xFF  │
└────────┴────────┴────────┴────────┴────────┘
  type      R        G        B        A

colorf (value: RGBA floats):
┌────────┬──── R (f32) ────┬──── G (f32) ────┬──── B (f32) ────┬──── A (f32) ────┐
│  0x14  │                 │                 │                 │                 │
└────────┴─────────────────┴─────────────────┴─────────────────┴─────────────────┘
```

### Matrix Types

Matrices are stored in column-major order:

```
mat3 (3×3 matrix):
┌────────┬─ col0 (3×f32) ─┬─ col1 (3×f32) ─┬─ col2 (3×f32) ─┐
│  0x15  │                │                │                │
└────────┴────────────────┴────────────────┴────────────────┘

mat4 (4×4 matrix):
┌────────┬─ col0 (4×f32) ─┬─ col1 (4×f32) ─┬─ col2 (4×f32) ─┬─ col3 (4×f32) ─┐
│  0x16  │                │                │                │                │
└────────┴────────────────┴────────────────┴────────────────┴────────────────┘
```

## Type Coercion

When receiving values, implementations should handle these coercions:

| From | To | Behavior |
|------|----|----------|
| i32 | i64 | Sign-extend |
| i32 | f64 | Convert exactly |
| i64 | f64 | Convert (may lose precision) |
| f64 | i32/i64 | Truncate (if in range) |

## Language Mappings

### Rust

```rust
use clasp_core::Value;

let v = Value::Float(0.75);
let v = Value::String("hello".into());
let v = Value::Array(vec![Value::Int(1), Value::Int(2)]);
let v = Value::Map(HashMap::from([
    ("x".into(), Value::Float(0.5)),
    ("y".into(), Value::Float(0.3)),
]));
```

### JavaScript

```javascript
// Values map to native JS types
client.set('/path', 0.75);           // f64
client.set('/path', 'hello');        // str
client.set('/path', [1, 2, 3]);      // array
client.set('/path', { x: 0.5 });     // map
client.set('/path', true);           // bool
client.set('/path', null);           // null
```

### Python

```python
# Values map to native Python types
await client.set('/path', 0.75)           # f64
await client.set('/path', 'hello')        # str
await client.set('/path', [1, 2, 3])      # array
await client.set('/path', {'x': 0.5})     # map
await client.set('/path', True)           # bool
await client.set('/path', None)           # null
```

## MessagePack Compatibility

For legacy compatibility, CLASP decoders also accept MessagePack encoding. The encoding is auto-detected by the first payload byte:

- MessagePack map prefix (0x80-0x8F, 0xDE, 0xDF): MessagePack encoding
- Otherwise: Binary encoding

MessagePack type mapping:

| MessagePack | CLASP |
|-------------|-------|
| nil | null |
| false/true | bool |
| int | i32 or i64 |
| float | f64 |
| str | str |
| bin | bin |
| array | array |
| map | map |
