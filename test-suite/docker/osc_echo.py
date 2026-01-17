#!/usr/bin/env python3
"""
OSC Echo Server for CLASP Testing

Listens on port 8000 and echoes messages back to the sender.
Also listens on port 9000 for general OSC messages.
"""

from pythonosc import dispatcher
from pythonosc import osc_server
from pythonosc import udp_client
from pythonosc import osc_message_builder
import threading
import socket

def echo_handler(client_address, address, *args):
    """Echo OSC message back to sender"""
    print(f"Received {address} {args} from {client_address}")

    # Extract sender IP and port
    if client_address:
        sender_ip = client_address[0]
        # Send back on port 9000 or try to parse from /ping
        sender_port = 9000

        # If it's a /ping message with return address
        if address == "/ping" and args:
            try:
                parts = str(args[0]).split(":")
                if len(parts) == 2:
                    sender_port = int(parts[1])
            except:
                pass

        try:
            client = udp_client.SimpleUDPClient(sender_ip, sender_port)
            # Echo back with /pong if it was /ping
            reply_address = "/pong" if address == "/ping" else address + "/echo"
            client.send_message(reply_address, list(args))
            print(f"  -> Sent {reply_address} to {sender_ip}:{sender_port}")
        except Exception as e:
            print(f"  -> Echo failed: {e}")

def default_handler(address, *args):
    """Log any unhandled OSC messages"""
    print(f"Unhandled: {address} {args}")

def run_server(port):
    """Run OSC server on specified port"""
    disp = dispatcher.Dispatcher()
    disp.map("/*", echo_handler, needs_reply_address=True)
    disp.set_default_handler(default_handler)

    server = osc_server.ThreadingOSCUDPServer(("0.0.0.0", port), disp)
    print(f"OSC Echo Server listening on port {port}")
    server.serve_forever()

if __name__ == "__main__":
    # Run servers on both ports
    t1 = threading.Thread(target=run_server, args=(8000,), daemon=True)
    t2 = threading.Thread(target=run_server, args=(9000,), daemon=True)

    t1.start()
    t2.start()

    print("OSC Echo Server running on ports 8000 and 9000")

    # Keep main thread alive
    try:
        while True:
            import time
            time.sleep(1)
    except KeyboardInterrupt:
        print("\nShutting down...")
