"""
CLASP P2P WebRTC Example (Python)

Demonstrates peer-to-peer communication using WebRTC DataChannels.

Usage:
    # Terminal 1 - Start as initiator
    PEER_ID=peer-a python p2p_webrtc.py

    # Terminal 2 - Start as responder
    PEER_ID=peer-b CONNECT_TO=peer-a python p2p_webrtc.py
"""

import asyncio
import os
import math
from clasp_to import Clasp, P2PManager

RENDEZVOUS_URL = os.environ.get("RENDEZVOUS_URL", "wss://rendezvous.clasp.to")
PEER_ID = os.environ.get("PEER_ID", f"peer-{int(asyncio.get_event_loop().time() * 1000)}")
CONNECT_TO = os.environ.get("CONNECT_TO")


async def main():
    print(f"\n=== CLASP P2P WebRTC Example (Python) ===")
    print(f"Peer ID: {PEER_ID}")

    # Create P2P manager
    p2p = P2PManager(
        peer_id=PEER_ID,
        rendezvous_url=RENDEZVOUS_URL,
        ice_servers=[
            {"urls": "stun:stun.l.google.com:19302"},
            {"urls": "stun:stun1.l.google.com:19302"},
        ],
        use_unreliable_channel=True,
    )

    # Handle incoming connections
    @p2p.on("connection")
    def on_connection(peer):
        print(f"[P2P] Peer connected: {peer.id}")

        @peer.on("/chat/*")
        def on_chat(value, address):
            print(f"[{peer.id}] {address}: {value}")

        @peer.on("/sensor/accel")
        def on_accel(value):
            print(
                f"[{peer.id}] Accelerometer: x={value['x']:.2f}, y={value['y']:.2f}, z={value['z']:.2f}"
            )

    @p2p.on("disconnection")
    def on_disconnection(peer_id):
        print(f"[P2P] Peer disconnected: {peer_id}")

    @p2p.on("error")
    def on_error(err):
        print(f"[P2P] Error: {err}")

    # Register with rendezvous server
    print(f"\nRegistering with rendezvous server...")
    await p2p.register(
        tags=["demo", "webrtc", "python"],
        metadata={"name": f"Python Peer {PEER_ID}", "capabilities": ["chat", "sensors"]},
    )
    print("Registered successfully!")

    # Connect to peer if specified
    if CONNECT_TO:
        print(f"\nConnecting to peer: {CONNECT_TO}...")
        try:
            peer = await p2p.connect(CONNECT_TO)
            print(f"Connected to {peer.id}!")

            # Send greeting
            await peer.set("/chat/greeting", f"Hello from Python {PEER_ID}!")

            # Stream sensor data
            async def stream_sensors():
                i = 0
                while True:
                    t = i * 0.1
                    await peer.stream(
                        "/sensor/accel",
                        {"x": math.sin(t), "y": math.cos(t), "z": math.sin(t * 0.5)},
                    )
                    i += 1
                    await asyncio.sleep(0.1)

            asyncio.create_task(stream_sensors())

        except Exception as e:
            print(f"Failed to connect: {e}")
    else:
        print(f"\nWaiting for incoming connections...")
        print(f"Run another instance with: CONNECT_TO={PEER_ID} python p2p_webrtc.py")

    # Discover peers
    print(f"\nDiscovering peers...")
    peers = await p2p.discover(tags=["demo"])
    print(f"Found {len(peers)} peer(s):")
    for p in peers:
        print(f"  - {p['id']}: {p.get('metadata', {}).get('name', 'Unknown')}")

    # Keep running
    print(f"\nPress Ctrl+C to exit")
    await asyncio.Event().wait()


if __name__ == "__main__":
    asyncio.run(main())
