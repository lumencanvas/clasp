"""
CLASP Bundles & Scheduling Example (Python)

Demonstrates atomic bundles and scheduled execution.

Usage:
    python bundles_and_scheduling.py
"""

import asyncio
import os
from datetime import datetime
from clasp_to import Clasp

CLASP_URL = os.environ.get("CLASP_URL", "ws://localhost:7330")


async def main():
    print("=== CLASP Bundles & Scheduling Example (Python) ===\n")

    client = Clasp(CLASP_URL)
    await client.connect(name="Bundle Demo")
    print("Connected to CLASP server")

    # Subscribe to see all changes
    @client.on("/**")
    def on_change(value, address, meta):
        time_str = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        print(f"[{time_str}] {address} = {value}")

    await asyncio.sleep(0.1)

    # =====================
    # 1. Atomic Bundle
    # =====================
    print("\n--- 1. Atomic Bundle ---")
    print("Setting multiple values atomically...")

    await client.bundle(
        [
            {"type": "set", "address": "/scene/active", "value": "sunset"},
            {"type": "set", "address": "/lights/1/brightness", "value": 0.8},
            {"type": "set", "address": "/lights/1/color", "value": {"r": 255, "g": 180, "b": 100}},
            {"type": "set", "address": "/lights/2/brightness", "value": 0.6},
            {"type": "emit", "address": "/scene/activated", "payload": {"name": "sunset"}},
        ]
    )
    print("Atomic bundle sent!")

    await asyncio.sleep(0.5)

    # =====================
    # 2. Scheduled Bundle
    # =====================
    print("\n--- 2. Scheduled Bundle ---")

    server_time = client.time()
    execute_at = server_time + 2_000_000  # 2 seconds from now
    print(f"Scheduling bundle for 2 seconds from now...")

    await client.bundle(
        [
            {"type": "set", "address": "/scheduled/counter", "value": 1},
            {"type": "emit", "address": "/scheduled/triggered", "payload": {"scheduled": True}},
        ],
        at=execute_at,
    )

    print("Scheduled bundle sent! Waiting for execution...")
    await asyncio.sleep(2.5)

    # =====================
    # 3. Chained Animation
    # =====================
    print("\n--- 3. Chained Scheduled Bundles (Animation) ---")
    print("Creating a 5-step fade animation...")

    animation_start = client.time() + 500_000
    step_duration = 200_000

    for i in range(6):
        brightness = i / 5
        execute_time = animation_start + (i * step_duration)

        await client.bundle(
            [
                {"type": "set", "address": "/animation/brightness", "value": brightness},
                {"type": "set", "address": "/animation/step", "value": i},
            ],
            at=execute_time,
        )

    print("Animation scheduled! Watching...")
    await asyncio.sleep(2.0)

    # =====================
    # 4. Mixed Bundle
    # =====================
    print("\n--- 4. Mixed Bundle with Events and Params ---")

    await client.bundle(
        [
            {"type": "set", "address": "/cue/current", "value": "intro"},
            {"type": "set", "address": "/cue/progress", "value": 0},
            {"type": "emit", "address": "/cue/started", "payload": {"name": "intro"}},
        ]
    )
    print("Mixed bundle sent!")

    await asyncio.sleep(0.5)

    await client.disconnect()
    print("\n=== Bundle demos complete ===")


if __name__ == "__main__":
    asyncio.run(main())
