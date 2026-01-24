"""
CLASP Security & Capability Tokens Example (Python)

Demonstrates authentication and authorization using CPSK tokens.

Usage:
    # Start router with security enabled
    cargo run -p clasp-router-server -- --security token --listen 0.0.0.0:7330

    # Run with token
    CLASP_TOKEN=cpsk_xxx python security_tokens.py
"""

import asyncio
import os
from clasp_to import Clasp

CLASP_URL = os.environ.get("CLASP_URL", "ws://localhost:7330")

# Example tokens (in production, generate via CLI)
TOKENS = {
    "read_only": os.environ.get("READ_TOKEN", "cpsk_read_only_demo_token"),
    "lights_control": os.environ.get("LIGHTS_TOKEN", "cpsk_lights_control_demo_token"),
    "admin": os.environ.get("ADMIN_TOKEN", "cpsk_admin_demo_token"),
}


async def demonstrate_read_only():
    print("\n=== Read-Only Token Demo ===")
    print("Scope: read:/**")

    client = Clasp(CLASP_URL)

    try:
        await client.connect(token=TOKENS["read_only"], name="Read-Only Client")
        print("Connected with read-only token")

        # Subscribe works
        @client.on("/lights/**")
        def on_lights(value, address, meta):
            print(f"[READ] {address} = {value}")

        print("Subscribed to /lights/** - OK")

        # Try to write (should fail)
        try:
            await client.set("/lights/1/brightness", 0.5)
            print("Write succeeded - UNEXPECTED!")
        except Exception as e:
            print(f"Write denied as expected: {e}")

        await client.disconnect()
    except Exception as e:
        print(f"Connection failed: {e}")


async def demonstrate_scoped_write():
    print("\n=== Scoped Write Token Demo ===")
    print("Scope: read:/**,write:/lights/**")

    client = Clasp(CLASP_URL)

    try:
        await client.connect(token=TOKENS["lights_control"], name="Lights Controller")
        print("Connected with lights control token")

        # Write to lights (allowed)
        await client.set("/lights/living-room/brightness", 0.75)
        print("Set /lights/living-room/brightness = 0.75 - OK")

        # Try to write outside namespace (should fail)
        try:
            await client.set("/audio/master/volume", 0.8)
            print("Write to /audio succeeded - UNEXPECTED!")
        except Exception as e:
            print(f"Write to /audio denied as expected: {e}")

        await client.disconnect()
    except Exception as e:
        print(f"Connection failed: {e}")


async def demonstrate_admin():
    print("\n=== Admin Token Demo ===")
    print("Scope: admin:/**")

    client = Clasp(CLASP_URL)

    try:
        await client.connect(token=TOKENS["admin"], name="Admin Client")
        print("Connected with admin token")

        # Admin can write everywhere
        await client.set("/lights/1/brightness", 1.0)
        await client.set("/audio/master/volume", 0.7)
        await client.set("/system/config/debug", True)
        print("Admin writes to all namespaces - OK")

        await client.disconnect()
    except Exception as e:
        print(f"Connection failed: {e}")


async def main():
    print("=== CLASP Security & Tokens Example (Python) ===")
    print(f"Server: {CLASP_URL}")

    # Check if server requires auth
    test_client = Clasp(CLASP_URL)
    try:
        await test_client.connect(name="Test")
        print("\nNote: Server is in OPEN mode (no authentication required)")
        await test_client.disconnect()
        return
    except Exception as e:
        if "AUTH_REQUIRED" in str(e):
            print("\nServer requires authentication - proceeding with demos")
        else:
            raise

    await demonstrate_read_only()
    await demonstrate_scoped_write()
    await demonstrate_admin()

    print("\n=== All security demos complete ===")


if __name__ == "__main__":
    asyncio.run(main())
