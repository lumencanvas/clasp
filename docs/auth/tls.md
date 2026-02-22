---
title: TLS Setup
description: Enable TLS encryption for CLASP connections
order: 5
---

# TLS Setup

TLS encrypts all traffic between clients and the relay. It is required for production deployments and mandatory for QUIC transport.

## Relay Configuration

Pass a PEM-encoded certificate and private key to the relay:

```bash
clasp-relay --cert /path/to/cert.pem --key /path/to/key.pem
```

Once TLS is enabled, clients connect via `wss://` instead of `ws://`.

## Let's Encrypt (Production)

For production deployments, use Caddy as a reverse proxy to get automatic TLS certificate issuance and renewal.

Caddyfile:

```
relay.yourdomain.com {
  reverse_proxy localhost:7330
}
```

Start Caddy and it handles certificates automatically. Clients connect to `wss://relay.yourdomain.com`.

This approach keeps the relay simple (no TLS termination) while providing trusted certificates with automatic renewal.

## Self-Signed (Development)

Generate a self-signed certificate with OpenSSL:

```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem \
  -days 365 -nodes -subj '/CN=localhost'
```

Start the relay:

```bash
clasp-relay --cert cert.pem --key key.pem
```

Note: clients must disable certificate verification when using self-signed certificates, since the certificate is not issued by a trusted CA.

## mkcert (Development)

mkcert creates locally-trusted development certificates. Your system and browsers will trust these certificates without any overrides.

```bash
brew install mkcert  # macOS; see mkcert docs for other platforms
mkcert -install
mkcert localhost 127.0.0.1 ::1
# Creates localhost+2.pem and localhost+2-key.pem
```

Start the relay with the generated files:

```bash
clasp-relay --cert localhost+2.pem --key localhost+2-key.pem
```

This is the recommended approach for local development since mkcert certificates are trusted by your system's certificate store without requiring clients to disable verification.

## Client Connection

All CLASP SDKs support TLS. Use a `wss://` URL instead of `ws://`.

**JavaScript:**

```javascript
const client = await new ClaspBuilder('wss://relay.yourdomain.com')
  .withToken('cpsk_abc123...')
  .connect();
```

**Python:**

```python
client = Clasp('wss://relay.yourdomain.com', token='cpsk_abc123...')
await client.connect()
```

**Rust:**

```rust
let client = Clasp::builder("wss://relay.yourdomain.com")
    .token("cpsk_abc123...")
    .connect().await?;
```

All SDKs auto-negotiate TLS when the `wss://` scheme is used.

## QUIC

QUIC transport requires TLS. Enable QUIC by specifying a port alongside the certificate and key:

```bash
clasp-relay --quic-port 7331 --cert cert.pem --key key.pem
```

QUIC provides lower latency than WebSocket for real-time applications, especially on unreliable networks, thanks to its built-in multiplexing and connection migration.

## Docker

Mount certificate files as read-only volumes:

```bash
docker run -p 7330:7330 \
  -v /path/to/certs:/certs:ro \
  clasp-relay --cert /certs/cert.pem --key /certs/key.pem
```

For Docker Compose:

```yaml
services:
  relay:
    image: clasp-relay
    ports:
      - "7330:7330"
    volumes:
      - ./certs:/certs:ro
    command: --cert /certs/cert.pem --key /certs/key.pem
```

## Troubleshooting

| Problem | Cause | Fix |
|---------|-------|-----|
| Certificate not trusted | Self-signed or unknown CA | Use a trusted CA, mkcert for dev, or disable verification in dev clients |
| Hostname mismatch | CN/SAN does not match connection URL | Regenerate cert with correct hostname in CN or SAN |
| Certificate expired | Cert past its validity period | Renew the certificate; use Caddy for auto-renewal |
| Connection refused on `wss://` | Relay started without TLS flags | Add `--cert` and `--key` flags to the relay |
| Mixed `ws://` and `wss://` | Client scheme does not match relay config | Use `wss://` when TLS is enabled, `ws://` when it is not |

## Next Steps

- [Auth Overview](../auth/README.md) -- token types and security modes
- [CPSK Tokens](cpsk.md) -- combine TLS with token authentication
