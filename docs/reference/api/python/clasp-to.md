# clasp-to (Python)

CLASP client library for Python.

## Overview

`clasp-to` provides an async CLASP client for Python 3.8+.

```bash
pip install clasp-to
```

## Quick Start

```python
import asyncio
from clasp import Clasp

async def main():
    client = await Clasp.connect('ws://localhost:7330')

    # Set a value
    await client.set('/sensors/temp', 23.5)

    # Get a value
    value = await client.get('/sensors/temp')
    print(f'Temperature: {value}')

    # Subscribe to changes
    @client.on('/sensors/**')
    async def on_sensor(value, address):
        print(f'{address}: {value}')

    # Keep running
    await client.run_forever()

asyncio.run(main())
```

## Connection

### Basic Connection

```python
client = await Clasp.connect('ws://localhost:7330')
```

### With Options

```python
client = await Clasp.connect(
    'ws://localhost:7330',
    name='my-python-client',
    timeout=10.0
)
```

### With Authentication

```python
client = await Clasp.connect(
    'wss://router.example.com:7330',
    token='eyJhbGciOi...'
)
```

### Auto-Discovery

```python
from clasp import discover_routers, Clasp

# Find all routers
routers = discover_routers(timeout=5.0)
for router in routers:
    print(f'{router.name}: {router.host}:{router.port}')

# Connect to first available
client = await Clasp.discover()

# Connect by name
client = await Clasp.discover(name='Studio Router')
```

### Auto-Reconnect

```python
client = await Clasp.connect(
    'ws://localhost:7330',
    auto_reconnect=True,
    reconnect_interval=1.0,
    max_reconnect_attempts=10
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

# With timeout
value = await client.get('/path', timeout=5.0)

# Returns None if not found
value = await client.get('/nonexistent')  # None
```

### emit(address, value)

```python
await client.emit('/events/button', {'button': 1})
await client.emit('/events/ping', None)
```

### delete(address)

```python
await client.delete('/path')
```

### list(pattern)

```python
addresses = await client.list('/sensors/**')
# ['/sensors/temp', '/sensors/humidity', ...]
```

## Subscriptions

### Decorator Style

```python
@client.on('/sensors/temp')
async def on_temp(value, address):
    print(f'Temperature: {value}')

@client.on('/sensors/**')
async def on_any_sensor(value, address):
    print(f'{address}: {value}')
```

### Method Style

```python
async def handler(value, address):
    print(f'{address}: {value}')

unsubscribe = client.on('/sensors/**', handler)

# Unsubscribe when done
unsubscribe()
```

### Options

```python
@client.on('/sensors/temp', max_rate=30, debounce=0.1, include_initial=True)
async def on_temp(value, address):
    print(value)
```

### Once

```python
value = await client.once('/events/ready')
```

## Bundles

```python
# List of operations
await client.bundle([
    {'set': ['/lights/1', 255]},
    {'set': ['/lights/2', 128]},
    {'emit': ['/cue/fired', {'cue': 1}]}
])

# Builder style
await (client.bundle()
    .set('/lights/1', 255)
    .set('/lights/2', 128)
    .emit('/cue/fired', {'cue': 1})
    .execute())
```

### Scheduled

```python
import time

# Execute 5 seconds from now
await (client.bundle()
    .set('/lights/1', 255)
    .at_time(time.time() + 5)
    .execute())
```

## Streams

```python
# Create stream
stream = client.stream('/audio/level')

# Send values
while True:
    level = get_audio_level()
    await stream.send(level)
    await asyncio.sleep(0.01)

# Stop
stream.stop()
```

## Gestures

```python
# Begin
gesture = client.gesture_begin('/draw/stroke', {'x': 100, 'y': 100})

# Update
await gesture.update({'x': 150, 'y': 120})
await gesture.update({'x': 200, 'y': 150})

# End
await gesture.end({'x': 250, 'y': 180})
```

## Connection Events

```python
@client.on_connected
async def connected():
    print('Connected')

@client.on_disconnected
async def disconnected(reason):
    print(f'Disconnected: {reason}')

@client.on_error
async def error(err):
    print(f'Error: {err}')
```

## Connection State

```python
# Check connection
if client.is_connected():
    # ...

# Wait for connection
await client.wait_connected()

# Ping
latency = await client.ping()
print(f'Latency: {latency}ms')
```

## Locks

```python
# Acquire lock
lock = await client.lock('/exclusive/resource')

try:
    await client.set('/exclusive/resource', value)
finally:
    await lock.release()

# Context manager
async with client.lock('/exclusive/resource'):
    await client.set('/exclusive/resource', value)
```

## Running

### Run Forever

```python
await client.run_forever()
```

### With Timeout

```python
try:
    await asyncio.wait_for(client.run_forever(), timeout=60.0)
except asyncio.TimeoutError:
    pass
```

### Graceful Shutdown

```python
import signal

async def main():
    client = await Clasp.connect('ws://localhost:7330')

    loop = asyncio.get_event_loop()

    def shutdown():
        asyncio.create_task(client.disconnect())

    loop.add_signal_handler(signal.SIGINT, shutdown)
    loop.add_signal_handler(signal.SIGTERM, shutdown)

    await client.run_forever()

asyncio.run(main())
```

## Type Hints

```python
from clasp import Clasp, Value
from typing import TypedDict

class SensorData(TypedDict):
    temp: float
    humidity: float

async def main():
    client = await Clasp.connect('ws://localhost:7330')

    data: SensorData = {'temp': 23.5, 'humidity': 65.0}
    await client.set('/sensors', data)

    result = await client.get('/sensors')
    # result is Value type
```

## Integration Examples

### FastAPI

```python
from fastapi import FastAPI
from clasp import Clasp

app = FastAPI()
client: Clasp = None

@app.on_event('startup')
async def startup():
    global client
    client = await Clasp.connect('ws://localhost:7330')

@app.on_event('shutdown')
async def shutdown():
    await client.disconnect()

@app.get('/api/value/{address:path}')
async def get_value(address: str):
    value = await client.get(f'/{address}')
    return {'value': value}

@app.post('/api/value/{address:path}')
async def set_value(address: str, body: dict):
    await client.set(f'/{address}', body['value'])
    return {'success': True}
```

### Django (Async)

```python
from django.http import JsonResponse
from clasp import Clasp

client = None

async def get_client():
    global client
    if client is None:
        client = await Clasp.connect('ws://localhost:7330')
    return client

async def get_value(request, address):
    c = await get_client()
    value = await c.get(f'/{address}')
    return JsonResponse({'value': value})
```

## Error Handling

```python
from clasp import ClaspError, NotFoundError, PermissionError, TimeoutError

try:
    value = await client.get('/path')
except NotFoundError:
    print('Not found')
except PermissionError:
    print('Access denied')
except TimeoutError:
    print('Timed out')
except ClaspError as e:
    print(f'Error: {e}')
```

## See Also

- [Python Installation](../../../how-to/installation/python-library.md)
- [Cross-Language Tutorial](../../../tutorials/cross-language-chat.md)
