# Cloud Deployment

Deploy CLASP routers in cloud environments.

## Docker

### Basic

```bash
docker run -p 7330:7330 lumencanvas/clasp-router
```

### Docker Compose

```yaml
version: '3.8'

services:
  clasp:
    image: lumencanvas/clasp-router
    ports:
      - "7330:7330"
    environment:
      - RUST_LOG=info
    restart: unless-stopped
```

### With TLS

```yaml
services:
  clasp:
    image: lumencanvas/clasp-router
    ports:
      - "7330:7330"
    volumes:
      - ./certs:/certs:ro
    environment:
      - CLASP_TLS_CERT=/certs/cert.pem
      - CLASP_TLS_KEY=/certs/key.pem
```

## Kubernetes

### Deployment

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
        image: lumencanvas/clasp-router:latest
        ports:
        - containerPort: 7330
        resources:
          requests:
            memory: "64Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "500m"
```

### Service

```yaml
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

### Ingress (WebSocket)

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: clasp-ingress
  annotations:
    nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "3600"
spec:
  rules:
  - host: clasp.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: clasp-router
            port:
              number: 7330
```

## Cloud Providers

### AWS

Using ECS:

```json
{
  "containerDefinitions": [{
    "name": "clasp-router",
    "image": "lumencanvas/clasp-router",
    "portMappings": [{
      "containerPort": 7330,
      "protocol": "tcp"
    }],
    "memory": 256
  }]
}
```

### Google Cloud Run

```bash
gcloud run deploy clasp-router \
  --image lumencanvas/clasp-router \
  --port 7330 \
  --allow-unauthenticated
```

Note: Cloud Run may have WebSocket limitations.

### DigitalOcean App Platform

```yaml
name: clasp
services:
- name: router
  image:
    registry_type: DOCKER_HUB
    registry: lumencanvas
    repository: clasp-router
  http_port: 7330
```

## Load Balancing

For high availability, use sticky sessions (WebSocket affinity):

### nginx

```nginx
upstream clasp {
  ip_hash;  # Sticky sessions
  server clasp1:7330;
  server clasp2:7330;
}

server {
  location / {
    proxy_pass http://clasp;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
  }
}
```

## Security

### TLS

Always use TLS in production:

```bash
docker run -p 7330:7330 \
  -v /path/to/certs:/certs \
  -e CLASP_TLS_CERT=/certs/fullchain.pem \
  -e CLASP_TLS_KEY=/certs/privkey.pem \
  lumencanvas/clasp-router
```

### Capability Tokens

Configure token validation:

```yaml
security:
  require_auth: true
  token_secret: ${TOKEN_SECRET}
```

### Firewall

Only expose necessary ports:
- 7330: WebSocket (clients)
- Internal only: QUIC (7331)

## Monitoring

### Health Check

```bash
curl http://localhost:7330/health
```

### Prometheus Metrics

```yaml
metrics:
  enabled: true
  port: 9090
```

### Logging

```yaml
environment:
  - RUST_LOG=info
```

Log levels: error, warn, info, debug, trace

## Scaling Considerations

### Single Router

Handles thousands of connections. Start here.

### Multiple Routers

For geographic distribution or redundancy:

```
Region A: clasp-a.example.com
Region B: clasp-b.example.com
```

Clients connect to nearest router. Cross-region sync is application-level.

## Next Steps

- [Enable TLS](../how-to/security/enable-tls.md)
- [Capability Tokens](../how-to/security/capability-tokens.md)
- [Performance Tuning](../how-to/advanced/performance-tuning.md)
