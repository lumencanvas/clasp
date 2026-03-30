#!/bin/bash
set -euo pipefail

# CLASP Relay bootstrap -- run on a fresh Ubuntu 22.04 droplet
# Usage: curl -fsSL https://raw.githubusercontent.com/lumencanvas/clasp/main/deploy/marketplace/digitalocean/bootstrap.sh | bash

REPO="https://raw.githubusercontent.com/lumencanvas/clasp/main/deploy/marketplace/digitalocean"

echo "==> Installing Docker"
apt-get update -qq
apt-get install -y -qq ca-certificates curl gnupg
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
chmod a+r /etc/apt/keyrings/docker.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" > /etc/apt/sources.list.d/docker.list
apt-get update -qq
DEBIAN_FRONTEND=noninteractive apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-compose-plugin
systemctl enable docker

echo "==> Pulling CLASP images"
docker pull ghcr.io/lumencanvas/clasp-relay:latest
docker pull caddy:2-alpine

echo "==> Installing clasp-setup"
mkdir -p /opt/clasp /var/lib/clasp
curl -fsSL "$REPO/files/clasp-setup" -o /usr/local/bin/clasp-setup
chmod +x /usr/local/bin/clasp-setup
curl -fsSL "$REPO/files/docker-compose.yml.tpl" -o /opt/clasp/docker-compose.yml.tpl
curl -fsSL "$REPO/files/Caddyfile.tpl" -o /opt/clasp/Caddyfile.tpl

echo "==> Installing MOTD"
mkdir -p /var/lib/digitalocean
curl -fsSL "$REPO/files/application.info" -o /var/lib/digitalocean/application.info
curl -fsSL "$REPO/files/99-one-click" -o /etc/update-motd.d/99-one-click
chmod +x /etc/update-motd.d/99-one-click
chmod -x /etc/update-motd.d/10-help-text 2>/dev/null || true
chmod -x /etc/update-motd.d/50-motd-news 2>/dev/null || true

echo "==> Configuring firewall"
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 443/udp
ufw allow 1883/tcp
ufw allow 8000/udp
ufw --force enable

echo ""
echo "  Done. Run 'clasp-setup' to configure your relay."
echo ""
