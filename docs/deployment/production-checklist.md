---
title: Production Checklist
description: Checklist for production CLASP deployments
order: 6
---

# Production Checklist

Before going to production, verify each item on this checklist. Not everything is required -- pick what applies to your use case. Items marked "Optional" are for specific feature sets.

## TLS

- [ ] TLS enabled via `--cert` and `--key` or a reverse proxy (Caddy, nginx)
- [ ] Clients connect via `wss://` not `ws://`
- [ ] Certificate auto-renewal configured (Let's Encrypt via Caddy, certbot, or cloud provider)
- [ ] QUIC port uses the same certificate: `--quic-port 7331 --cert cert.pem --key key.pem`

## Authentication

- [ ] Auth HTTP server enabled: `--auth-port 7350`
- [ ] CORS restricted to your domain: `--cors-origin https://yourdomain.com`
- [ ] Admin token file created and secured: `--admin-token /secure/admin.token`
- [ ] Admin token file has restricted permissions (`chmod 600`)
- [ ] Token TTL configured appropriately: `--token-ttl 86400`
- [ ] Optional: capability tokens enabled: `--trust-anchor anchor.pub`
- [ ] Optional: cap delegation depth limited: `--cap-max-depth 5`
- [ ] Optional: entity registry enabled: `--registry-db ./registry.db`

## Persistence

- [ ] State snapshots enabled: `--persist ./state.db`
- [ ] Snapshot interval configured: `--persist-interval 30`
- [ ] Persistent storage survives container/server restarts (Docker volume, cloud disk)
- [ ] Optional: journal for full event log: `--journal ./journal.db`
- [ ] Optional: in-memory journal for testing: `--journal-memory`

## App Config

- [ ] Application config loaded: `--app-config config/myapp.json`
- [ ] Write rules tested against your access patterns
- [ ] Snapshot visibility rules configured
- [ ] Config files placed in `/etc/clasp/*.json` or `./config/*.json` for auto-detection

## Limits

- [ ] Parameter TTL configured: `--param-ttl 3600`
- [ ] Signal TTL configured: `--signal-ttl 3600`
- [ ] Maximum sessions set: `--max-sessions 500`
- [ ] Session timeout configured: `--session-timeout 300`
- [ ] Drain timeout set for graceful shutdown: `--drain-timeout 30`

## Monitoring

- [ ] Log level set to `info` (not `debug` or `trace` in production)
- [ ] Structured logging enabled: `LOG_FORMAT=json`
- [ ] Optional: metrics endpoint exposed: `--metrics-port 9090`
- [ ] Health check configured (WebSocket TCP probe or HTTP auth health endpoint)
- [ ] Log aggregation configured (journald, Docker logging driver, or cloud logging)

## Networking

- [ ] Relay ports (7330, 7350) not exposed directly -- traffic goes through reverse proxy
- [ ] Firewall configured: allow 80/443, block direct relay ports
- [ ] DNS records point to the server or load balancer
- [ ] WebSocket upgrade headers forwarded by reverse proxy / load balancer

## Optional Features

- [ ] Federation configured: `--federation-hub`, `--federation-id`, `--federation-namespace`
- [ ] Federation token secured: `--federation-token`
- [ ] Rules engine loaded: `--rules ./rules.json`
- [ ] MQTT bridge enabled: `--mqtt-port 1883`
- [ ] OSC bridge enabled: `--osc-port 8000`
- [ ] Rendezvous server configured: `--rendezvous-port 7340 --rendezvous-ttl 300`

## Example Production Command

A complete production relay command incorporating the checklist items:

```bash
clasp-relay \
  --auth-port 7350 \
  --cors-origin https://app.yourdomain.com \
  --admin-token ./secrets/admin.token \
  --token-ttl 86400 \
  --journal ./data/journal.db \
  --persist ./data/state.db \
  --persist-interval 30 \
  --app-config ./config/app.json \
  --param-ttl 3600 \
  --signal-ttl 3600 \
  --max-sessions 500 \
  --session-timeout 300 \
  --drain-timeout 30
```

With environment:

```bash
RUST_LOG=info LOG_FORMAT=json clasp-relay \
  --auth-port 7350 \
  --cors-origin https://app.yourdomain.com \
  --admin-token ./secrets/admin.token \
  --journal ./data/journal.db \
  --persist ./data/state.db \
  --persist-interval 30 \
  --app-config ./config/app.json \
  --param-ttl 3600 \
  --max-sessions 500 \
  --admin-token ./secrets/admin.token
```

## Next Steps

- [Auth overview](../auth/README.md) -- configure authentication and pairing
- [App Config](../reference/router-config.md) -- configure application rules
- [TLS setup](../auth/tls.md) -- detailed certificate configuration
