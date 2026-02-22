---
title: "clasp-to (Python)"
description: "CLASP client library for Python."
section: reference
order: 1
---
# clasp-to (Python)

CLASP client library for Python.

## Overview

`clasp-to` provides an async CLASP client for Python 3.9+.

```bash
pip install clasp-to
```

## Quick Start

```python
import asyncio
from clasp import ClaspBuilder

async def main():
    client = await (
        ClaspBuilder('ws://localhost:7330')
        .with_name('My App')
        .connect()
    )

    # Set a value
    await client.set('/sensors/temp', 23.5)

    # Get a value
    value = await client.get('/sensors/temp')
    print(f'Temperature: {value}')

    # Subscribe to changes
    @client.on('/sensors/**')
    def on_sensor(value, address):
        print(f'{address}: {value}')

    # Keep running
    await asyncio.Event().wait()

asyncio.run(main())
```

## Connection

### Builder Pattern

```python
client = await (
    ClaspBuilder('ws://localhost:7330')
    .with_name('my-python-client')
    .connect()
)
```

### With Authentication

```python
client = await (
    ClaspBuilder('wss://router.example.com:7330')
    .with_name('my-client')
    .with_token('eyJhbGciOi...')
    .connect()
)
```

### Auto-Reconnect

```python
client = await (
    ClaspBuilder('ws://localhost:7330')
    .with_name('my-client')
    .with_reconnect(True, interval=1.0)
    .connect()
)
```

## Core Operations

### set(address, value)

```python
# Primitives
await client.set('/path', 42)
await client.set('/path', 'hello')
await client.set('/path', True)
await client.set('/path', 3.14)

# Dict
await client.set('/path', {'x': 1, 'y': 2})

# List
await client.set('/path', [1, 2, 3])

# None
await client.set('/path', None)
```

### get(address)

```python
value = await client.get('/path')

# Returns None if not found
value = await client.get('/nonexistent')  # None
```

### emit(address, value)

```python
await client.emit('/events/button', {'button': 1})
await client.emit('/events/ping')
```

## Subscriptions

### Decorator Style

```python
@client.on('/sensors/temp')
def on_temp(value, address):
    print(f'Temperature: {value}')

@client.on('/sensors/**')
def on_any_sensor(value, address):
    print(f'{address}: {value}')
```

### Method Style

```python
def handler(value, address):
    print(f'{address}: {value}')

unsubscribe = client.on('/sensors/**', handler)

# Unsubscribe when done
unsubscribe()
```

## Bundles

```python
# List of operations
await client.bundle([
    {'set': ['/lights/1', 255]},
    {'set': ['/lights/2', 128]},
    {'emit': ['/cue/fired', {'cue': 1}]}
])
```

## Connection Events

```python
@client.on_connected
def connected():
    print('Connected')

@client.on_disconnected
def disconnected():
    print('Disconnected')

@client.on_error
def error(err):
    print(f'Error: {err}')
```

## Connection State

```python
# Check connection
if client.connected:
    pass

# Get session ID
print(f'Session: {client.session}')
```

## Disconnect

```python
await client.close()
```

## Graceful Shutdown

```python
import asyncio
import signal
from clasp import ClaspBuilder

async def main():
    client = await (
        ClaspBuilder('ws://localhost:7330')
        .with_name('My App')
        .connect()
    )

    loop = asyncio.get_event_loop()

    def shutdown():
        asyncio.create_task(client.close())

    loop.add_signal_handler(signal.SIGINT, shutdown)
    loop.add_signal_handler(signal.SIGTERM, shutdown)

    await asyncio.Event().wait()

asyncio.run(main())
```

## Type Hints

Full type hints are included (PEP 561):

```python
from clasp import ClaspBuilder, Clasp

async def main() -> None:
    client: Clasp = await ClaspBuilder('ws://localhost:7330').connect()
    value: float = await client.get('/path')
```

## Integration Examples

### FastAPI

```python
from fastapi import FastAPI
from clasp import ClaspBuilder, Clasp

app = FastAPI()
client: Clasp = None

@app.on_event('startup')
async def startup():
    global client
    client = await (
        ClaspBuilder('ws://localhost:7330')
        .with_name('FastAPI Bridge')
        .connect()
    )

@app.on_event('shutdown')
async def shutdown():
    await client.close()

@app.get('/api/value/{address:path}')
async def get_value(address: str):
    value = await client.get(f'/{address}')
    return {'value': value}

@app.post('/api/value/{address:path}')
async def set_value(address: str, body: dict):
    await client.set(f'/{address}', body['value'])
    return {'success': True}
```

## Error Handling

```python
try:
    value = await client.get('/path')
except Exception as e:
    print(f'Error: {e}')
```

## See Also

- [Python Installation](../../../how-to/installation/python-library.md)
- [Cross-Language Tutorial](../../../tutorials/cross-language-chat.md)
