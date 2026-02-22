# CLASP Chat

Real-time chat app built on the CLASP protocol. Vue 3 SPA with text channels, video calls, friend system, and end-to-end encryption.

## Requirements

- Node.js 20+
- A running CLASP relay server (see [Relay Deployment](#relay))

## Local development

```bash
npm install
npm run dev
```

The app defaults to `wss://relay.clasp.chat` and `https://relay.clasp.chat` for the relay and auth API. Override with environment variables:

```bash
VITE_RELAY_URL=ws://localhost:7330 VITE_AUTH_API_URL=http://localhost:7350 npm run dev
```

## Build

```bash
npm run build
```

Output goes to `dist/`. It's a static SPA — serve it from anywhere.

To build with custom relay URLs:

```bash
VITE_AUTH_API_URL=https://relay.clasp.chat VITE_RELAY_URL=wss://relay.clasp.chat npm run build
```

## Deploy the SPA

The built `dist/` folder is plain static files. Deploy it wherever you host static sites:

- **Cloudflare Pages**: connect your repo, set build command to `npm run build`, output dir `dist`, add `VITE_AUTH_API_URL` and `VITE_RELAY_URL` as build environment variables
- **Vercel**: same as above
- **Netlify**: same as above, add a `_redirects` file with `/* /index.html 200` for SPA routing
- **Nginx/Caddy**: serve `dist/` with a fallback to `index.html` for client-side routing
- **Docker**: use the included `Dockerfile` which builds the SPA and serves it via nginx

### Docker

```bash
docker build \
  --build-arg VITE_AUTH_API_URL=https://relay.clasp.chat \
  --build-arg VITE_RELAY_URL=wss://relay.clasp.chat \
  -t clasp-chat .

docker run -p 8080:80 clasp-chat
```

## Relay

The chat app needs a CLASP relay server to connect to. The relay handles WebSocket connections, message routing, and user authentication.

### Use the public relay

By default the app connects to `relay.clasp.chat`. No setup needed — just build and deploy the SPA.

### Self-host a relay

See [`deploy/droplet/`](../../deploy/droplet/) for a full guide on deploying the relay to a DigitalOcean Droplet with automatic HTTPS, persistent auth database, and block storage.

The short version:
1. Create a Droplet + Block Storage Volume on DigitalOcean
2. Point `relay.yourdomain.com` DNS at the Droplet
3. Run the setup script and `docker compose up`
4. Build the SPA with your relay URLs and deploy it

### Run a relay locally

For local dev without the public relay:

```bash
cd deploy/relay
cargo run --features full -- --auth-port 7350
```

The relay auto-detects `config/chat.json` for chat-specific security rules (room creator checks, DM friendship validation, snapshot privacy). You can also run via Docker Compose:

```bash
cd deploy/relay
docker compose up
```

This starts a relay on `ws://localhost:7330` with auth on `http://localhost:7350`. Run the chat app with:

```bash
VITE_RELAY_URL=ws://localhost:7330 VITE_AUTH_API_URL=http://localhost:7350 npm run dev
```

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_RELAY_URL` | `wss://relay.clasp.chat` | WebSocket URL for the CLASP relay |
| `VITE_AUTH_API_URL` | `https://relay.clasp.chat` | HTTP URL for the auth API (login/register) |

Both are baked in at build time via Vite.
