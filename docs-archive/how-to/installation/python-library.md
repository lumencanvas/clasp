---
title: "Python Library"
description: "Add CLASP to Python projects."
section: how-to
order: 3
---
# Python Library

Add CLASP to Python projects.

## Install

```bash
pip install clasp-to
```

**Requirements:**
- Python 3.9 or later

## Usage

### Basic

```python
import asyncio
from clasp import ClaspBuilder

async def main():
    client = await (
        ClaspBuilder('ws://localhost:7330')
        .with_name('My App')
        .connect()
    )

    await client.set('/path', 42)
    await client.close()

asyncio.run(main())
```

### With Subscriptions

```python
import asyncio
from clasp import ClaspBuilder

async def main():
    client = await ClaspBuilder('ws://localhost:7330').connect()

    @client.on('/sensors/**')
    def on_sensor(value, address):
        print(f'{address} = {value}')

    # Keep running
    await client.run()

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

## Virtual Environments

```bash
# Create venv
python -m venv venv
source venv/bin/activate  # Linux/macOS
# or: venv\Scripts\activate  # Windows

# Install
pip install clasp-to
```

## Dependencies

- `websockets` — WebSocket client
- `msgpack` — Message encoding

## Next Steps

- [Connect a Client](../connections/connect-client.md)
- [Subscribe to Changes](../state/subscribe-to-changes.md)
- [Python API Reference](../../reference/api/python/clasp-to.md)
