#!/bin/bash
set -euo pipefail

RELAY_VERSION="${RELAY_VERSION:-latest}"

echo "==> Setting up CLASP Relay (${RELAY_VERSION})"

# Create directories
mkdir -p /opt/clasp /var/lib/clasp

# Pull the relay image so first boot is fast
docker pull "ghcr.io/lumencanvas/clasp-relay:${RELAY_VERSION}"
docker pull caddy:2-alpine

# Install marketplace files
cp /tmp/clasp-files/clasp-setup /usr/local/bin/clasp-setup
chmod +x /usr/local/bin/clasp-setup

cp /tmp/clasp-files/docker-compose.yml.tpl /opt/clasp/docker-compose.yml.tpl
cp /tmp/clasp-files/Caddyfile.tpl /opt/clasp/Caddyfile.tpl

# DigitalOcean marketplace metadata
mkdir -p /var/lib/digitalocean
cp /tmp/clasp-files/application.info /var/lib/digitalocean/application.info

# MOTD for SSH login
cp /tmp/clasp-files/99-one-click /etc/update-motd.d/99-one-click
chmod +x /etc/update-motd.d/99-one-click

# Disable default MOTD noise
chmod -x /etc/update-motd.d/10-help-text 2>/dev/null || true
chmod -x /etc/update-motd.d/50-motd-news 2>/dev/null || true

# Clean up temp files
rm -rf /tmp/clasp-files

echo "    CLASP Relay configured."
