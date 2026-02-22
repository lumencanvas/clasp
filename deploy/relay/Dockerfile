# CLASP Relay Server Dockerfile
#
# This is a STANDALONE deployment that uses published crates from crates.io.
# No monorepo dependencies required.
#
# Build:
#   docker build -t clasp-relay .
#
# Run:
#   docker run -p 7330:7330 clasp-relay

FROM rust:1.85-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only the relay source (standalone)
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

# Strip [patch.crates-io] section (only used for local dev, not Docker builds)
RUN sed -i '/^\[patch\.crates-io\]/,$d' Cargo.toml

# Build the relay binary
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/clasp-relay /usr/local/bin/clasp-relay

# Copy app config files
COPY config /etc/clasp/

# Create non-root user and data directory
RUN useradd -r -s /bin/false clasp && \
    mkdir -p /data && chown clasp:clasp /data
WORKDIR /data
USER clasp

# WebSocket port
EXPOSE 7330
# Rendezvous API port
EXPOSE 7340

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -sf http://localhost:7360/healthz || exit 1

ENTRYPOINT ["clasp-relay"]
CMD ["--ws-port", "7330", "--auth-port", "7350", "--health-port", "7360", \
     "--app-config", "/etc/clasp/chat.json", "--name", "relay.clasp.to"]
