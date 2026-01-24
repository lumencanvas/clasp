"""
CLASP Late Joiner Synchronization Example (Python)

Demonstrates how late-joining clients receive current state.
Key differentiator from OSC which has no state sync.

Usage:
    python late_joiner.py
"""

import asyncio
import os
from clasp_to import Clasp

CLASP_URL = os.environ.get("CLASP_URL", "ws://localhost:7330")


async def main():
    print("=== CLASP Late Joiner Synchronization Example (Python) ===\n")

    # =====================
    # Setup: Initialize state
    # =====================
    print("--- Setup: Initializing State ---")

    initializer = Clasp(CLASP_URL)
    await initializer.connect(name="State Initializer")

    print("Setting up initial state...\n")

    await initializer.set("/lights/living-room/brightness", 0.8)
    await initializer.set("/lights/living-room/color", {"r": 255, "g": 240, "b": 220})
    await initializer.set("/lights/kitchen/brightness", 1.0)
    await initializer.set("/lights/bedroom/brightness", 0.3)

    await initializer.set("/audio/master/volume", 0.65)
    await initializer.set("/audio/master/mute", False)

    await initializer.set("/scene/active", "evening")

    print("Initial state created with 7 params")
    print("Disconnecting initializer...\n")
    await initializer.disconnect()

    await asyncio.sleep(0.5)

    # =====================
    # 1. Late Joiner
    # =====================
    print("--- 1. Late Joiner with Full Wildcard ---")

    late_joiner = Clasp(CLASP_URL)
    await late_joiner.connect(name="Late Joiner")

    print("New client connected. Subscribing to /**...\n")

    snapshot_count = [0]

    @late_joiner.on("/**")
    def on_snapshot(value, address, meta):
        snapshot_count[0] += 1
        rev = meta.get("revision", "?")
        print(f"  [SNAPSHOT] {address} = {value} (rev: {rev})")

    await asyncio.sleep(0.3)
    print(f"\nReceived {snapshot_count[0]} params in snapshot")

    await late_joiner.disconnect()

    # =====================
    # 2. Selective Subscription
    # =====================
    print("\n--- 2. Selective Subscription ---")

    lights_only = Clasp(CLASP_URL)
    await lights_only.connect(name="Lights Only")

    print("Subscribing to /lights/** only...\n")

    lights_count = [0]

    @lights_only.on("/lights/**")
    def on_lights(value, address, meta):
        lights_count[0] += 1
        print(f"  [SNAPSHOT] {address} = {value}")

    await asyncio.sleep(0.2)
    print(f"\nReceived {lights_count[0]} light params (audio/scene excluded)")

    await lights_only.disconnect()

    # =====================
    # OSC vs CLASP comparison
    # =====================
    print("\n--- OSC vs CLASP Comparison ---")

    print("\nWith OSC (stateless):")
    print("  - Client connects")
    print("  - Client waits for someone to send updates...")
    print("  - No initial state available")

    print("\nWith CLASP (stateful):")
    print("  - Client connects")
    print("  - Subscribes to patterns")
    print("  - IMMEDIATELY receives ALL current values")

    # Demonstrate instant sync
    demo = Clasp(CLASP_URL)
    await demo.connect(name="Instant Sync Demo")

    import time

    start_time = time.time()
    received = [0]

    @demo.on("/**")
    def on_demo(value, address, meta):
        received[0] += 1
        if received[0] == 1:
            elapsed = (time.time() - start_time) * 1000
            print(f"\n  First param received in {elapsed:.1f}ms after connect!")

    await asyncio.sleep(0.3)
    print(f"  Total: {received[0]} params synced instantly")

    await demo.disconnect()

    print("\n=== Late joiner demo complete ===")
    print("\nKey takeaway: Unlike OSC, CLASP clients get current state immediately.")


if __name__ == "__main__":
    asyncio.run(main())
