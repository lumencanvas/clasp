# Publishing CLASP to Package Managers

## Pre-Publishing Checklist

- [ ] `cargo test --workspace` passes
- [ ] `cargo test --features full` passes in `deploy/relay/`
- [ ] `cargo clippy --workspace` clean (or acceptable warnings)
- [ ] `cargo fmt --all -- --check` passes
- [ ] Version bumped in root `Cargo.toml` (workspace version)
- [ ] Version bumped in `deploy/relay/Cargo.toml` + dep pins match workspace
- [ ] Version bumped in `bindings/js/packages/clasp-sdk/package.json`
- [ ] CHANGELOG updated (if maintained)

---

## Cargo (crates.io)

### Setup (One-time)

```bash
cargo login <your-api-token>
```

Get token from https://crates.io/settings/tokens

### Publish Order

Publish in dependency order. Wait ~1 minute between each for crates.io to index.

```bash
# Layer 0: No internal dependencies
cargo publish -p clasp-core

# Layer 1: Depends on clasp-core only
cargo publish -p clasp-transport
cargo publish -p clasp-discovery
cargo publish -p clasp-journal
cargo publish -p clasp-bridge
cargo publish -p clasp-crypto
cargo publish -p clasp-identity
cargo publish -p clasp-caps
cargo publish -p clasp-registry
cargo publish -p clasp-rules
cargo publish -p clasp-lens
cargo publish -p clasp-embedded

# Layer 2: Depends on Layer 0-1
cargo publish -p clasp-router
cargo publish -p clasp-client
cargo publish -p clasp-federation
cargo publish -p clasp-test-utils

# Layer 3: DefraDB crates (depend on Layer 1)
cargo publish -p clasp-journal-defra
cargo publish -p clasp-state-defra
cargo publish -p clasp-defra-bridge
cargo publish -p clasp-defra-transport
cargo publish -p clasp-registry-defra
cargo publish -p clasp-config-defra

# Layer 4: Bindings
cargo publish -p clasp-wasm
cargo publish -p clasp-cli
```

### Dry Run

```bash
cargo publish -p clasp-core --dry-run
```

---

## npm (@clasp-to)

### Setup (One-time)

```bash
npm login
# Must be a member of the @clasp-to org on npmjs.com
```

### Publish

```bash
# Core protocol
cd bindings/js/packages/clasp-core
npm run build
npm publish --access public

# Encryption
cd ../clasp-crypto
npm run build
npm publish --access public

# High-level SDK (includes registry, journal, identity clients)
cd ../clasp-sdk
npm run build
npm publish --access public

# Relay wrapper
cd ../clasp-relay
npm run build
npm publish --access public
```

---

## PyPI (clasp-to)

```bash
cd bindings/python
pip install build twine
python -m build
twine upload dist/*
```

---

## Docker Image (ghcr.io)

```bash
cd deploy/relay
docker build --build-arg FEATURES=full -t ghcr.io/lumencanvas/clasp-relay:4.4.0 .
docker tag ghcr.io/lumencanvas/clasp-relay:4.4.0 ghcr.io/lumencanvas/clasp-relay:latest
docker push ghcr.io/lumencanvas/clasp-relay:4.4.0
docker push ghcr.io/lumencanvas/clasp-relay:latest
```

---

## Relay Binary (clasp-relay)

The relay is a standalone binary at `deploy/relay/`, not part of the cargo workspace. Publish separately:

```bash
cd deploy/relay
cargo publish
```

Note: The relay's `[patch.crates-io]` section must be removed before publishing. The `Cargo.toml` dep pins (e.g., `clasp-core = "4.4"`) must match published versions.

---

## DigitalOcean Marketplace Image

```bash
cd deploy/marketplace/digitalocean
export DIGITALOCEAN_TOKEN="your-token"
packer init .
packer build template.pkr.hcl
```

See `deploy/marketplace/digitalocean/README.md` for the full submission process.

---

## GitHub Release

```bash
git tag v4.4.0
git push --tags
```

The CI workflow builds and attaches desktop app binaries (macOS, Windows, Linux) to the release automatically.

---

## Package Names

| Manager | Package | Install |
|---------|---------|---------|
| Cargo | `clasp-cli` | `cargo install clasp-cli` |
| Cargo | `clasp-core` | `cargo add clasp-core` |
| Cargo | `clasp-router` | `cargo add clasp-router` |
| Cargo | `clasp-lens` | `cargo add clasp-lens` |
| Cargo | `clasp-relay` | `cargo install clasp-relay` |
| npm | `@clasp-to/core` | `npm install @clasp-to/core` |
| npm | `@clasp-to/sdk` | `npm install @clasp-to/sdk` |
| npm | `@clasp-to/crypto` | `npm install @clasp-to/crypto` |
| PyPI | `clasp-to` | `pip install clasp-to` |
| Docker | `ghcr.io/lumencanvas/clasp-relay` | `docker pull ghcr.io/lumencanvas/clasp-relay` |
| DO Marketplace | CLASP Relay | 1-Click Droplet |
