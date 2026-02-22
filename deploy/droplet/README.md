# CLASP Relay — Droplet Deployment

Deploys the CLASP relay server on a DigitalOcean Droplet with:

- **Caddy** for automatic HTTPS (Let's Encrypt, zero config)
- **Block storage volume** for persistent user data (auth database)
- **Docker Compose** for orchestration

## What's in the box

| Container | Purpose |
|-----------|---------|
| `caddy` | Reverse proxy, auto HTTPS termination |
| `relay` | CLASP multi-protocol relay + HTTP auth API (uses `chat.json` for app-specific rules) |

The relay exposes these ports:
- **7330** — WebSocket (CLASP protocol, proxied through Caddy with TLS)
- **7350** — HTTP auth API (`/auth/register`, `/auth/login`, proxied through Caddy)
- **1883** — MQTT (exposed directly, raw TCP)
- **8000** — OSC (exposed directly, UDP)

Caddy routes both through a single HTTPS domain.

## Prerequisites

- A DigitalOcean account
- A domain with DNS you can control (e.g. `relay.clasp.to`)

## Deploy steps

### 1. Create the Droplet

In the DO console:
- **Image**: Ubuntu 22.04+
- **Size**: Basic, 1GB RAM ($6/mo) is enough
- **Region**: wherever your users are
- **Add SSH key** for access

### 2. Create and attach a Block Storage Volume

In the DO console under **Volumes**:
- **Name**: `clasp_data` (or whatever you want, the setup script takes it as an argument)
- **Size**: 1GB is plenty (auth DB is tiny)
- **Attach to**: your Droplet

This volume persists your user database across container rebuilds and Droplet recreations.

### 3. Point DNS

Create an A record pointing your relay domain to the Droplet's IP:

```
A  relay.clasp.to  →  <droplet-ip>
```

### 4. Clone and run setup

SSH into the Droplet:

```bash
ssh root@<droplet-ip>

git clone <your-repo-url> /opt/clasp
cd /opt/clasp/deploy/droplet

# Pass your volume name if not "clasp_data"
bash setup.sh clasp_data
```

The setup script:
- Installs Docker + Compose
- Formats and mounts the block storage volume
- Adds it to `/etc/fstab` so it survives reboots
- Creates the data directory structure
- Copies `.env.example` to `.env`

### 5. Configure .env

```bash
nano .env
```

Set your relay domain and a real email for Let's Encrypt:

```env
RELAY_DOMAIN=relay.clasp.to
ACME_EMAIL=you@example.com
DATA_DIR=/mnt/clasp_data
```

### 6. Start

```bash
docker compose up -d --build
```

That's it. Caddy automatically provisions an HTTPS certificate from Let's Encrypt on first request. No cert files, no cron jobs, no renewal scripts. It just works as long as:
- Port 80 and 443 are open (DO firewall)
- DNS points at the Droplet

Check logs:
```bash
docker compose logs -f
```

## HTTPS / Let's Encrypt

Caddy handles everything automatically:
- Provisions certs on first request via ACME (Let's Encrypt)
- Renews before expiry (no cron needed)
- Redirects HTTP to HTTPS
- HTTP/2 and HTTP/3 (QUIC) enabled by default

**Requirements**: ports 80 + 443 open, DNS A record pointing at the Droplet. That's all.

## Persistent data

The auth database is stored on the block storage volume at:

```
/mnt/clasp_data/relay/chat-auth.db
```

The compose file passes `--auth-db /data/chat-auth.db` and `--app-config /etc/clasp/chat.json` to the relay. The app config is baked into the Docker image; the auth database lives on the volume.

This file contains registered usernames and Argon2 password hashes. It survives:
- Container rebuilds (`docker compose up --build`)
- Docker volume pruning (it's a bind mount, not a Docker volume)
- Droplet power cycles

It also survives Droplet destruction as long as you don't delete the DO volume — you can detach it and reattach to a new Droplet.

CLASP state (rooms, messages, presence) lives in relay memory and is **not** persisted to disk. The relay runs with `--no-ttl` so state stays alive as long as the process runs, but a relay restart clears it.

## Updating

```bash
cd /opt/clasp
git pull
cd deploy/droplet
docker compose up -d --build
```

## Firewall

If using the DO cloud firewall, allow:
- **80/tcp** — HTTP (needed for Let's Encrypt ACME challenge)
- **443/tcp** — HTTPS (WebSocket + auth API)
- **443/udp** — HTTP/3 (optional)
- **1883/tcp** — MQTT (bridge protocol)
- **8000/udp** — OSC (bridge protocol, unicast only over WAN)
- **22/tcp** — SSH
