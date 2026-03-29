#!/bin/bash
set -euo pipefail

echo "==> Configuring firewall (ufw)"

ufw default deny incoming
ufw default allow outgoing

ufw allow 22/tcp    comment "SSH"
ufw allow 80/tcp    comment "HTTP (Caddy redirect)"
ufw allow 443/tcp   comment "HTTPS (Caddy TLS)"
ufw allow 443/udp   comment "HTTP/3 QUIC"
ufw allow 1883/tcp  comment "MQTT bridge"
ufw allow 8000/udp  comment "OSC bridge"

ufw --force enable
echo "    Firewall configured."
