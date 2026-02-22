---
title: Cloud Deployment
description: Deploy CLASP relay to cloud providers
order: 5
---

# Cloud Deployment

This guide walks through deploying a CLASP relay to DigitalOcean. The approach adapts to any cloud provider that supports Docker or has a Rust build toolchain.

## DigitalOcean Droplet

A single droplet is sufficient for most CLASP deployments. The relay is lightweight -- a 1GB RAM droplet handles hundreds of concurrent sessions.

### 1. Create the Droplet

Create an Ubuntu 22.04 droplet with at least 1GB RAM. Enable the firewall and add your SSH key.

```bash
doctl compute droplet create clasp-relay \
  --image ubuntu-22-04-x64 \
  --size s-1vcpu-1gb \
  --region nyc1 \
  --ssh-keys <your-key-id>
```

### 2. Install Dependencies

SSH into the droplet and install Rust:

```bash
ssh root@<droplet-ip>

apt update && apt install -y build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

### 3. Build the Relay

Clone the repository and build:

```bash
git clone https://github.com/your-org/clasp.git
cd clasp/deploy/relay
cargo build --release --features full
cp target/release/clasp-relay /usr/local/bin/
```

Alternatively, install directly from crates.io:

```bash
cargo install clasp-relay --features full
```

### 4. Create a Systemd Service

Create the data directories and service file:

```bash
mkdir -p /var/lib/clasp /etc/clasp
```

Write the service file to `/etc/systemd/system/clasp-relay.service`:

```ini
[Unit]
Description=CLASP Relay Server
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/clasp-relay \
  --auth-port 7350 \
  --cors-origin https://app.yourdomain.com \
  --admin-token /etc/clasp/admin.token \
  --journal /var/lib/clasp/journal.db \
  --persist /var/lib/clasp/state.db \
  --persist-interval 30 \
  --param-ttl 3600 \
  --max-sessions 500
Environment=RUST_LOG=info
Environment=LOG_FORMAT=json
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Generate an admin token and enable the service:

```bash
openssl rand -hex 32 > /etc/clasp/admin.token
chmod 600 /etc/clasp/admin.token

systemctl daemon-reload
systemctl enable clasp-relay
systemctl start clasp-relay
```

Check status:

```bash
systemctl status clasp-relay
journalctl -u clasp-relay -f
```

### 5. Configure TLS with Caddy

Install Caddy for automatic TLS via Let's Encrypt:

```bash
apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list
apt update && apt install caddy
```

Write the Caddyfile to `/etc/caddy/Caddyfile`:

```
relay.yourdomain.com {
    reverse_proxy localhost:7330
}

auth.yourdomain.com {
    reverse_proxy localhost:7350
}
```

Restart Caddy:

```bash
systemctl restart caddy
```

Caddy automatically provisions and renews TLS certificates.

### 6. DNS

Create A records pointing to the droplet IP:

| Type | Name    | Value          |
| ---- | ------- | -------------- |
| A    | relay   | `<droplet-ip>` |
| A    | auth    | `<droplet-ip>` |

### 7. Firewall

Allow HTTP/HTTPS for Caddy and block direct access to relay ports:

```bash
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw deny 7330/tcp
ufw deny 7350/tcp
ufw enable
```

Clients connect via `wss://relay.yourdomain.com` -- Caddy handles TLS and proxies to the relay.

## DigitalOcean App Platform

For a managed deployment without server administration, use App Platform with the relay Dockerfile:

```bash
doctl apps create --spec deploy/relay/digitalocean/app.yaml
```

App Platform handles scaling, TLS, and container management. Point your spec at the `deploy/relay/` directory and configure environment variables for relay flags.

## Other Providers

The relay runs on any host with Docker or a Rust toolchain. Key requirements for any provider:

- **Persistent storage** -- the journal, auth database, and state snapshots must survive restarts. Use a persistent volume or managed database.
- **TLS termination** -- either via a reverse proxy (Caddy, nginx, cloud load balancer) or the relay's built-in TLS (`--cert`, `--key`).
- **WebSocket support** -- load balancers and reverse proxies must support WebSocket upgrades. Most do by default, but verify connection upgrade headers are forwarded.
- **Sticky sessions** -- not required. The relay is a single process, so any connection reaches the same instance.

**AWS:** Use ECS with Fargate and an ALB. Mount an EFS volume for persistence. The ALB handles TLS and WebSocket upgrades.

**Fly.io:** Deploy via Dockerfile. Fly supports persistent volumes and automatic TLS.

**Bare metal:** Build with `cargo build --release --features full`, run behind Caddy or nginx, use systemd for process management.

## Next Steps

- [Production Checklist](production-checklist.md) -- verify your cloud deployment
- [TLS setup](../auth/tls.md) -- detailed TLS configuration
