#!/usr/bin/env bash
set -euo pipefail

# CLASP Relay — DigitalOcean Droplet Setup
#
# Prerequisites:
#   1. Create a Droplet (Ubuntu 22.04+, 1GB+ RAM)
#   2. Create a Block Storage Volume named "clasp_data" and attach it
#   3. Point DNS: A relay.clasp.to → <droplet-ip>
#   4. SSH in and run this script
#
# Usage:
#   cd deploy/droplet && bash setup.sh

VOLUME_NAME="${1:-clasp_data}"
MOUNT_POINT="/mnt/${VOLUME_NAME}"

echo "==> Installing Docker"
if ! command -v docker &>/dev/null; then
  apt-get update
  apt-get install -y ca-certificates curl gnupg
  install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
  chmod a+r /etc/apt/keyrings/docker.gpg
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
    https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" \
    > /etc/apt/sources.list.d/docker.list
  apt-get update
  apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
  systemctl enable --now docker
  echo "    Docker installed."
else
  echo "    Docker already installed."
fi

echo "==> Mounting block storage volume"
DISK="/dev/disk/by-id/scsi-0DO_Volume_${VOLUME_NAME}"
if [ ! -e "$DISK" ]; then
  echo "    WARNING: Volume device $DISK not found."
  echo "    Make sure you created and attached a volume named '${VOLUME_NAME}' in the DO console."
  echo "    Falling back to local directory ${MOUNT_POINT}"
  mkdir -p "${MOUNT_POINT}"
else
  mkdir -p "${MOUNT_POINT}"
  if ! blkid "$DISK" &>/dev/null; then
    echo "    Formatting volume..."
    mkfs.ext4 -F "$DISK"
  fi
  if ! mountpoint -q "${MOUNT_POINT}"; then
    mount -o defaults,nofail,discard "$DISK" "${MOUNT_POINT}"
  fi
  if ! grep -q "${VOLUME_NAME}" /etc/fstab; then
    echo "$DISK ${MOUNT_POINT} ext4 defaults,nofail,discard 0 2" >> /etc/fstab
  fi
  echo "    Volume mounted at ${MOUNT_POINT}"
fi

echo "==> Creating data directories"
mkdir -p "${MOUNT_POINT}/relay"

echo "==> Setting up .env"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [ ! -f "${SCRIPT_DIR}/.env" ]; then
  cp "${SCRIPT_DIR}/.env.example" "${SCRIPT_DIR}/.env"
  echo "    Created .env from .env.example"
  echo ""
  echo "    Edit it:  nano ${SCRIPT_DIR}/.env"
  echo "    Then run: cd ${SCRIPT_DIR} && docker compose up -d --build"
else
  echo "    .env already exists."
  echo "    Run: cd ${SCRIPT_DIR} && docker compose up -d --build"
fi
