"""
SignalFlow - Universal Creative Protocol

A modern protocol for creative tools communication, bridging OSC, MIDI, DMX,
Art-Net and more with a unified, stateful interface.

Example:
    >>> import asyncio
    >>> from signalflow import SignalFlow
    >>>
    >>> async def main():
    ...     sf = SignalFlow('ws://localhost:7330')
    ...     await sf.connect()
    ...
    ...     @sf.on('/lumen/layer/*/opacity')
    ...     def on_opacity(value, address):
    ...         print(f'{address} = {value}')
    ...
    ...     await sf.set('/lumen/layer/0/opacity', 0.75)
    ...     sf.run()
    >>>
    >>> asyncio.run(main())
"""

__version__ = "0.1.0"
__author__ = "LumenCanvas"

from .client import SignalFlow, SignalFlowBuilder
from .types import (
    Value,
    SignalType,
    QoS,
    Message,
    PROTOCOL_VERSION,
    DEFAULT_WS_PORT,
    DEFAULT_DISCOVERY_PORT,
)

__all__ = [
    "SignalFlow",
    "SignalFlowBuilder",
    "Value",
    "SignalType",
    "QoS",
    "Message",
    "PROTOCOL_VERSION",
    "DEFAULT_WS_PORT",
    "DEFAULT_DISCOVERY_PORT",
]
