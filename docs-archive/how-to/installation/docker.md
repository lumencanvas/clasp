---
title: "Docker"
description: "Run CLASP in Docker containers."
section: how-to
order: 5
---
# Docker

Run CLASP in Docker containers.

## Quick Start

```bash
docker run -p 7330:7330 lumencanvas/clasp-router
```

## Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  clasp-router:
    image: lumencanvas/clasp-router
    ports:
      - "7330:7330"
    restart: unless-stopped

  # Optional: MQTT broker for IoT integration
  mqtt:
    image: eclipse-mosquitto
    ports:
      - "1883:1883"

  # Optional: MQTT bridge
  clasp-mqtt:
    image: lumencanvas/clasp-cli
    command: mqtt --host mqtt --port 1883 --router ws://clasp-router:7330
    depends_on:
      - clasp-router
      - mqtt
```

Start:
```bash
docker compose up -d
```

## Build from Source

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p clasp-cli

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/clasp /usr/local/bin/
EXPOSE 7330
CMD ["clasp", "server", "--bind", "0.0.0.0:7330"]
```

## Configuration

### Environment Variables

```yaml
services:
  clasp-router:
    image: lumencanvas/clasp-router
    environment:
      - RUST_LOG=info
      - CLASP_PORT=7330
      - CLASP_BIND=0.0.0.0
```

### Volume for Persistence

```yaml
services:
  clasp-router:
    volumes:
      - clasp-data:/data
    environment:
      - CLASP_STATE_DIR=/data

volumes:
  clasp-data:
```

## Networking

### Bridge Network (Default)

Services communicate by name:

```yaml
# clasp-mqtt connects to "clasp-router:7330"
```

### Host Network

For protocol bridges that need host access:

```yaml
services:
  clasp-midi:
    network_mode: host
    command: midi --device "Launchpad"
```

## Health Checks

```yaml
services:
  clasp-router:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:7330/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## Kubernetes

Basic deployment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: clasp-router
spec:
  replicas: 1
  selector:
    matchLabels:
      app: clasp-router
  template:
    metadata:
      labels:
        app: clasp-router
    spec:
      containers:
      - name: clasp
        image: lumencanvas/clasp-router
        ports:
        - containerPort: 7330
---
apiVersion: v1
kind: Service
metadata:
  name: clasp-router
spec:
  selector:
    app: clasp-router
  ports:
  - port: 7330
    targetPort: 7330
  type: ClusterIP
```

## Next Steps

- [Start a Router](../connections/start-router.md)
- [Cloud Deployment Guide](../../use-cases/cloud-deployment.md)
