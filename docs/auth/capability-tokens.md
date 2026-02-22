---
title: Capability Tokens
description: Ed25519 delegatable capability tokens for CLASP
order: 3
---

# Capability Tokens

Capability tokens are Ed25519-signed, delegatable tokens inspired by UCAN. They enable offline token issuance and hierarchical scope delegation without contacting the relay. A root key holder can mint tokens and delegate subsets of their access to others, forming a verifiable chain of authority.

## Enable

Capability token support requires the `caps` feature at build time:

```bash
cargo build --release --features caps
```

At runtime, specify one or more trust anchors (the public keys whose signatures the relay will accept as roots):

```bash
clasp-relay --trust-anchor ./root.pub
```

Multiple trust anchors are supported:

```bash
clasp-relay --trust-anchor a.pub --trust-anchor b.pub
```

## Generate Keys

Create an Ed25519 keypair for signing tokens:

```bash
clasp key generate --out root.key
clasp key show root.key
# Output: ed25519 public key in hex
```

The private key is saved to `root.key`. The public key (shown by `key show`) is what you pass to `--trust-anchor` on the relay.

## Create a Root Token

Mint a token signed by a trust anchor key:

```bash
clasp token cap create \
  --key root.key \
  --scopes "write:/lights/**" \
  --expires 30d
# Output: cap_<base64url...>
```

This token grants `write` access (which implies `read`) to all paths under `/lights/` and expires in 30 days.

## Delegate a Token

A token holder can delegate a subset of their access to another key:

```bash
# Generate a child key
clasp key generate --out child.key

# Delegate with narrowed scope
clasp token cap delegate cap_<parent-token> \
  --key child.key \
  --scopes "write:/lights/zone1/**" \
  --expires 7d
# Output: cap_<child-token>
```

The child token is independently verifiable. No server round-trip is needed to create it.

## Delegation Rules

Each delegation in the chain must satisfy these constraints:

| Rule | Description |
|------|-------------|
| Scope subset | Child scopes must be a subset of parent scopes |
| Action narrowing | Child can narrow actions (e.g., `write` to `read`) but never widen |
| Expiration | Child expiration must be less than or equal to parent expiration |
| Chain depth | Maximum delegation depth is 5 by default |

The action hierarchy is `admin > write > read`. A parent with `write:/lights/**` can delegate `read:/lights/**` or `write:/lights/zone1/**`, but cannot delegate `admin:/lights/**` or `write:/sensors/**`.

Configure the maximum chain depth:

```bash
clasp-relay --trust-anchor root.pub --cap-max-depth 3
```

## Inspect and Verify

Examine a token's contents without verifying the chain:

```bash
clasp token cap inspect cap_<token>
# Shows: issuer, audience, scopes, expiration, chain depth
```

Verify the entire delegation chain against a trust anchor:

```bash
clasp token cap verify cap_<token> --trust-anchor root.key
# Verifies: root issuer matches anchor, all signatures valid,
#           scopes narrow correctly, nothing expired, depth within limit
```

## Using Capability Tokens

Connect to a relay with a capability token the same way as any other token type:

**JavaScript:**

```javascript
const client = await new ClaspBuilder('ws://localhost:7330')
  .withToken('cap_<token>')
  .connect();
```

**Python:**

```python
client = Clasp('ws://localhost:7330', token='cap_<token>')
await client.connect()
```

**Rust:**

```rust
let client = Clasp::builder("ws://localhost:7330")
    .token("cap_<token>")
    .connect().await?;
```

## How It Works

When a client presents a `cap_` token, the relay validates the entire delegation chain:

1. The root issuer's public key matches a configured trust anchor.
2. Each link in the chain has a valid Ed25519 signature.
3. Each delegation narrows (or maintains) the parent's scopes.
4. No link in the chain has expired.
5. The chain depth does not exceed the configured maximum.

If any check fails, the connection is rejected. The token is encoded as base64url-encoded MessagePack containing the full chain of proofs.

## Use Cases

**IoT device provisioning.** A factory holds the root key and mints per-device tokens offline. Each device gets a token scoped to its own namespace (e.g., `write:/devices/sensor-42/**`). No network access to the relay is needed during provisioning.

**Temporary access delegation.** An admin delegates a short-lived token to a contractor with access limited to specific paths and a 24-hour expiration.

**Third-party integrations.** Grant a visualization service read-only access to sensor data (`read:/sensors/**`) without exposing write capabilities or other namespaces.

## Next Steps

- [Entity Registry](entity-registry.md) -- persistent identity for devices and services
- [CPSK Tokens](cpsk.md) -- simpler register/login/guest authentication
- [Security Model](../concepts/security-model.md) -- deep dive into CLASP's security architecture
