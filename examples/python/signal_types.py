"""
CLASP Signal Types Example (Python)

Demonstrates all five signal types:
- Param: Stateful values
- Event: One-shot triggers
- Stream: High-frequency data
- Gesture: Phased input
- Timeline: Automation

Usage:
    python signal_types.py
"""

import asyncio
import os
import math
from clasp_to import Clasp

CLASP_URL = os.environ.get("CLASP_URL", "ws://localhost:7330")


async def main():
    print("=== CLASP Signal Types Example (Python) ===\n")

    client = Clasp(CLASP_URL)
    await client.connect(name="Signal Types Demo")
    print("Connected to CLASP server")

    # =====================
    # 1. PARAM
    # =====================
    print("\n--- 1. PARAM (Stateful Values) ---")

    @client.on("/mixer/**")
    def on_mixer(value, address, meta):
        print(f"[PARAM] {address} = {value} (rev: {meta.get('revision', '?')})")

    await asyncio.sleep(0.1)

    await client.set("/mixer/master/volume", 0.8)
    await client.set("/mixer/master/mute", False)
    await client.set("/mixer/channel/1/volume", 0.65)

    # Get cached value
    master_vol = client.value("/mixer/master/volume")
    print(f"\nCached master volume: {master_vol}")

    await asyncio.sleep(0.3)

    # =====================
    # 2. EVENT
    # =====================
    print("\n--- 2. EVENT (One-Shot Triggers) ---")

    @client.on_event("/cue/**")
    def on_cue(payload, address):
        print(f"[EVENT] {address}: {payload}")

    await asyncio.sleep(0.1)

    await client.emit("/cue/go", {"cue_id": "intro", "fade_time": 2.0})
    await client.emit("/cue/stop", {"immediate": False})

    await asyncio.sleep(0.3)

    # =====================
    # 3. STREAM
    # =====================
    print("\n--- 3. STREAM (High-Frequency Data) ---")

    stream_count = [0]

    @client.on_stream("/sensor/**", max_rate=30)
    def on_sensor(value, address):
        stream_count[0] += 1
        if stream_count[0] % 10 == 0:
            print(f"[STREAM] {address} = {value} (received {stream_count[0]})")

    await asyncio.sleep(0.1)

    # Stream at 60Hz for 1 second
    print("Streaming 60 values...")
    for i in range(60):
        t = i / 60
        await client.stream(
            "/sensor/accelerometer",
            {"x": math.sin(t * math.pi * 2), "y": math.cos(t * math.pi * 2), "z": 0.98},
        )
        await asyncio.sleep(0.016)

    print(f"Sent 60 stream values, received {stream_count[0]}")

    await asyncio.sleep(0.3)

    # =====================
    # 4. GESTURE
    # =====================
    print("\n--- 4. GESTURE (Phased Input) ---")

    @client.on_gesture("/input/**")
    def on_gesture(gesture):
        phase = gesture.get("phase", "?")
        x = gesture.get("x", 0)
        y = gesture.get("y", 0)
        print(f"[GESTURE] {phase.upper()} at ({x:.0f}, {y:.0f})")

    await asyncio.sleep(0.1)

    gesture_id = f"drag-{int(asyncio.get_event_loop().time() * 1000)}"

    await client.gesture("/input/mouse", {"id": gesture_id, "phase": "start", "x": 100, "y": 100})

    for i in range(1, 6):
        await client.gesture(
            "/input/mouse", {"id": gesture_id, "phase": "move", "x": 100 + i * 20, "y": 100 + i * 10}
        )
        await asyncio.sleep(0.05)

    await client.gesture("/input/mouse", {"id": gesture_id, "phase": "end", "x": 200, "y": 150})

    await asyncio.sleep(0.3)

    # =====================
    # 5. TIMELINE
    # =====================
    print("\n--- 5. TIMELINE (Automation) ---")

    @client.on_timeline("/automation/**")
    def on_timeline(timeline, address):
        kf_count = len(timeline.get("keyframes", []))
        duration = timeline.get("duration", 0)
        print(f"[TIMELINE] {address}: {kf_count} keyframes, duration={duration}ms")

    await asyncio.sleep(0.1)

    await client.timeline(
        "/automation/light/brightness",
        {
            "duration": 5000,
            "loop": True,
            "keyframes": [
                {"time": 0, "value": 0.0, "easing": "linear"},
                {"time": 1000, "value": 1.0, "easing": "ease-out"},
                {"time": 3000, "value": 1.0, "easing": "linear"},
                {"time": 5000, "value": 0.0, "easing": "ease-in"},
            ],
        },
    )

    await asyncio.sleep(0.3)

    # Summary
    print("\n=== Signal Type Summary ===")
    print("| Type     | QoS     | Persists | Use Case                    |")
    print("|----------|---------|----------|-----------------------------|")
    print("| Param    | Confirm | Yes      | Faders, settings, state     |")
    print("| Event    | Confirm | No       | Button press, cue trigger   |")
    print("| Stream   | Fire    | No       | Sensors, meters (30-60Hz)   |")
    print("| Gesture  | Fire    | No       | Touch, pen, mouse drag      |")
    print("| Timeline | Commit  | Yes      | Animation, automation       |")

    await client.disconnect()
    print("\n=== Signal types demo complete ===")


if __name__ == "__main__":
    asyncio.run(main())
