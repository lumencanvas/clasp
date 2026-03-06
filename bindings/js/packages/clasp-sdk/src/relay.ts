import type { RelayConfig, AppConfig } from './types'

/** Validate that a port number is in the valid range. */
function validatePort(port: number, label: string): void {
  if (!Number.isInteger(port) || port < 0 || port > 65535) {
    throw new RangeError(`${label} must be an integer between 0 and 65535, got ${port}`)
  }
}

/**
 * Fluent builder for generating relay CLI commands, Docker Compose configs,
 * and environment variables for CLASP relay deployment.
 *
 * Does not run the relay -- generates the configuration artifacts.
 *
 * @example
 * ```typescript
 * const relay = new RelayBuilder()
 *   .port(7330)
 *   .authPort(7350)
 *   .corsOrigin('https://app.example.com')
 *   .persist('./state.db')
 *   .appConfig({ scopes: ['read:/**'] })
 *
 * relay.toCommand()         // CLI command string
 * relay.toDockerCompose()   // Docker Compose YAML
 * relay.toEnv()             // env var format
 * relay.toAppConfigJSON()   // app config as JSON string
 * relay.toRulesJSON()       // rules as JSON string
 * ```
 */
export class RelayBuilder {
  private config: RelayConfig = {}

  /** Create a RelayBuilder from an existing config object. */
  static fromConfig(config: RelayConfig): RelayBuilder {
    const builder = new RelayBuilder()
    builder.config = { ...config }
    return builder
  }

  /** WebSocket listener port (default: 7330). */
  port(port: number): this {
    validatePort(port, 'port')
    this.config.port = port
    return this
  }

  /** Bind address (default: 0.0.0.0). */
  host(host: string): this {
    this.config.host = host
    return this
  }

  /** Human-readable relay name. */
  name(name: string): this {
    this.config.name = name
    return this
  }

  /** Enable auth HTTP server on this port. */
  authPort(port: number): this {
    validatePort(port, 'authPort')
    this.config.authPort = port
    return this
  }

  /** Set allowed CORS origins. */
  corsOrigin(origins: string | string[]): this {
    this.config.corsOrigin = origins
    return this
  }

  /** Path to admin token file. */
  adminTokenPath(path: string): this {
    this.config.adminTokenPath = path
    return this
  }

  /** Token lifetime in seconds. */
  tokenTtl(seconds: number): this {
    this.config.tokenTtl = seconds
    return this
  }

  /** Maximum concurrent sessions. */
  maxSessions(n: number): this {
    this.config.maxSessions = n
    return this
  }

  /** Idle session timeout in seconds. */
  sessionTimeout(seconds: number): this {
    this.config.sessionTimeout = seconds
    return this
  }

  /** Parameter TTL in seconds. */
  paramTtl(seconds: number): this {
    this.config.paramTtl = seconds
    return this
  }

  /** Signal TTL in seconds. */
  signalTtl(seconds: number): this {
    this.config.signalTtl = seconds
    return this
  }

  /** Disable all TTL expiration. */
  noTtl(): this {
    this.config.noTtl = true
    return this
  }

  /** Enable verbose logging. */
  verbose(): this {
    this.config.verbose = true
    return this
  }

  /** Set log level (error, warn, info, debug, trace). */
  logLevel(level: 'error' | 'warn' | 'info' | 'debug' | 'trace'): this {
    this.config.logLevel = level
    return this
  }

  /** Enable state persistence. */
  persist(path: string, options?: { interval?: number }): this {
    this.config.persist = { path, interval: options?.interval }
    return this
  }

  /** Enable journal (append-only log). Use 'memory' for in-memory ring buffer. */
  journal(pathOrMemory: string | 'memory', options?: { batchSize?: number; flushMs?: number }): this {
    if (pathOrMemory === 'memory') {
      this.config.journal = 'memory'
    } else {
      this.config.journal = { path: pathOrMemory, ...options }
    }
    return this
  }

  /** TLS certificate and key paths. */
  tls(cert: string, key: string): this {
    this.config.cert = cert
    this.config.key = key
    return this
  }

  /** Set app config (scopes, write rules, visibility). */
  appConfig(config: AppConfig | string): this {
    this.config.appConfig = config
    return this
  }

  /** Set rules engine config. */
  rules(rules: Record<string, unknown> | string): this {
    this.config.rules = rules
    return this
  }

  /** Add capability token trust anchors. */
  capabilityTokens(options: { trustAnchors: string[]; maxDepth?: number }): this {
    this.config.trustAnchors = options.trustAnchors
    if (options.maxDepth !== undefined) this.config.capMaxDepth = options.maxDepth
    return this
  }

  /** Enable entity registry. */
  entityRegistry(dbPath: string): this {
    this.config.registryDb = dbPath
    return this
  }

  /** Enable MQTT server. */
  mqtt(port: number, options?: { namespace?: string }): this {
    validatePort(port, 'mqtt port')
    this.config.mqtt = { port, namespace: options?.namespace }
    return this
  }

  /** Enable OSC server. */
  osc(port: number, options?: { namespace?: string }): this {
    validatePort(port, 'osc port')
    this.config.osc = { port, namespace: options?.namespace }
    return this
  }

  /** Enable QUIC transport. */
  quic(port: number, cert: string, key: string): this {
    validatePort(port, 'quic port')
    this.config.quic = { port, cert, key }
    return this
  }

  /** Configure federation (leaf mode). */
  federation(options: { hub: string; id: string; namespaces: string[]; token?: string }): this {
    this.config.federation = options
    return this
  }

  /** Enable rendezvous/discovery. */
  rendezvous(options?: { port?: number; ttl?: number }): this {
    this.config.rendezvous = options || {}
    return this
  }

  /** Graceful shutdown timeout in seconds. */
  drainTimeout(seconds: number): this {
    this.config.drainTimeout = seconds
    return this
  }

  /** Enable health check HTTP server on this port (serves /healthz and /readyz). */
  healthPort(port: number): this {
    validatePort(port, 'healthPort')
    this.config.healthPort = port
    return this
  }

  /**
   * Merge another builder's config into this one.
   * Values from `other` override this builder where both are set.
   */
  merge(other: RelayBuilder): this {
    const otherConfig = other.getConfig()
    this.config = { ...this.config, ...otherConfig }
    return this
  }

  /** Get the raw config object. */
  getConfig(): RelayConfig {
    return { ...this.config }
  }

  /**
   * Generate CLI arguments as an array, suitable for `child_process.spawn()`.
   * Does not include the binary name -- only the flags and values.
   */
  toArgs(): string[] {
    const c = this.config
    const args: string[] = []

    if (c.port) args.push('--ws-port', String(c.port))
    if (c.host) args.push('--host', c.host)
    if (c.name) args.push('--name', c.name)
    if (c.authPort) args.push('--auth-port', String(c.authPort))
    if (c.corsOrigin) {
      const origins = Array.isArray(c.corsOrigin) ? c.corsOrigin.join(',') : c.corsOrigin
      args.push('--cors-origin', origins)
    }
    if (c.adminTokenPath) args.push('--admin-token', c.adminTokenPath)
    if (c.tokenTtl) args.push('--token-ttl', String(c.tokenTtl))
    if (c.maxSessions) args.push('--max-sessions', String(c.maxSessions))
    if (c.sessionTimeout) args.push('--session-timeout', String(c.sessionTimeout))
    if (c.paramTtl) args.push('--param-ttl', String(c.paramTtl))
    if (c.signalTtl) args.push('--signal-ttl', String(c.signalTtl))
    if (c.noTtl) args.push('--no-ttl')
    if (c.logLevel) {
      args.push('--log-level', c.logLevel)
    } else if (c.verbose) {
      args.push('--verbose')
    }

    if (c.persist) {
      args.push('--persist', c.persist.path)
      if (c.persist.interval) args.push('--persist-interval', String(c.persist.interval))
    }

    if (c.journal) {
      if (c.journal === 'memory') {
        args.push('--journal-memory')
      } else {
        args.push('--journal', c.journal.path)
        if (c.journal.batchSize) args.push('--journal-batch-size', String(c.journal.batchSize))
        if (c.journal.flushMs) args.push('--journal-flush-ms', String(c.journal.flushMs))
      }
    }

    if (c.cert) args.push('--cert', c.cert)
    if (c.key) args.push('--key', c.key)

    if (typeof c.appConfig === 'string') {
      args.push('--app-config', c.appConfig)
    }

    if (typeof c.rules === 'string') {
      args.push('--rules', c.rules)
    }

    if (c.trustAnchors) {
      for (const anchor of c.trustAnchors) {
        args.push('--trust-anchor', anchor)
      }
    }
    if (c.capMaxDepth) args.push('--cap-max-depth', String(c.capMaxDepth))
    if (c.registryDb) args.push('--registry-db', c.registryDb)

    if (c.mqtt) {
      args.push('--mqtt-port', String(c.mqtt.port))
      if (c.mqtt.namespace) args.push('--mqtt-namespace', c.mqtt.namespace)
    }
    if (c.osc) {
      args.push('--osc-port', String(c.osc.port))
      if (c.osc.namespace) args.push('--osc-namespace', c.osc.namespace)
    }
    if (c.quic) {
      args.push('--quic-port', String(c.quic.port))
      if (!c.cert) args.push('--cert', c.quic.cert)
      if (!c.key) args.push('--key', c.quic.key)
    }

    if (c.federation) {
      args.push('--federation-hub', c.federation.hub)
      args.push('--federation-id', c.federation.id)
      for (const ns of c.federation.namespaces) {
        args.push('--federation-namespace', ns)
      }
      if (c.federation.token) args.push('--federation-token', c.federation.token)
    }

    if (c.rendezvous) {
      if (c.rendezvous.port) args.push('--rendezvous-port', String(c.rendezvous.port))
      if (c.rendezvous.ttl) args.push('--rendezvous-ttl', String(c.rendezvous.ttl))
    }

    if (c.drainTimeout) args.push('--drain-timeout', String(c.drainTimeout))
    if (c.healthPort) args.push('--health-port', String(c.healthPort))

    return args
  }

  /** Generate the CLI command string. */
  toCommand(): string {
    const c = this.config
    const parts = ['clasp-relay']

    if (c.port) parts.push(`--ws-port ${c.port}`)
    if (c.host) parts.push(`--host ${c.host}`)
    if (c.name) parts.push(`--name ${JSON.stringify(c.name)}`)
    if (c.authPort) parts.push(`--auth-port ${c.authPort}`)
    if (c.corsOrigin) {
      const origins = Array.isArray(c.corsOrigin) ? c.corsOrigin.join(',') : c.corsOrigin
      parts.push(`--cors-origin ${origins}`)
    }
    if (c.adminTokenPath) parts.push(`--admin-token ${c.adminTokenPath}`)
    if (c.tokenTtl) parts.push(`--token-ttl ${c.tokenTtl}`)
    if (c.maxSessions) parts.push(`--max-sessions ${c.maxSessions}`)
    if (c.sessionTimeout) parts.push(`--session-timeout ${c.sessionTimeout}`)
    if (c.paramTtl) parts.push(`--param-ttl ${c.paramTtl}`)
    if (c.signalTtl) parts.push(`--signal-ttl ${c.signalTtl}`)
    if (c.noTtl) parts.push('--no-ttl')
    if (c.logLevel) {
      parts.push(`--log-level ${c.logLevel}`)
    } else if (c.verbose) {
      parts.push('--verbose')
    }

    if (c.persist) {
      parts.push(`--persist ${c.persist.path}`)
      if (c.persist.interval) parts.push(`--persist-interval ${c.persist.interval}`)
    }

    if (c.journal) {
      if (c.journal === 'memory') {
        parts.push('--journal-memory')
      } else {
        parts.push(`--journal ${c.journal.path}`)
        if (c.journal.batchSize) parts.push(`--journal-batch-size ${c.journal.batchSize}`)
        if (c.journal.flushMs) parts.push(`--journal-flush-ms ${c.journal.flushMs}`)
      }
    }

    if (c.cert) parts.push(`--cert ${c.cert}`)
    if (c.key) parts.push(`--key ${c.key}`)

    if (typeof c.appConfig === 'string') {
      parts.push(`--app-config ${c.appConfig}`)
    } else if (c.appConfig) {
      parts.push('--app-config ./app-config.json')
    }

    if (typeof c.rules === 'string') {
      parts.push(`--rules ${c.rules}`)
    } else if (c.rules) {
      parts.push('--rules ./rules.json')
    }

    if (c.trustAnchors) {
      for (const anchor of c.trustAnchors) {
        parts.push(`--trust-anchor ${anchor}`)
      }
    }
    if (c.capMaxDepth) parts.push(`--cap-max-depth ${c.capMaxDepth}`)
    if (c.registryDb) parts.push(`--registry-db ${c.registryDb}`)

    if (c.mqtt) {
      parts.push(`--mqtt-port ${c.mqtt.port}`)
      if (c.mqtt.namespace) parts.push(`--mqtt-namespace ${c.mqtt.namespace}`)
    }
    if (c.osc) {
      parts.push(`--osc-port ${c.osc.port}`)
      if (c.osc.namespace) parts.push(`--osc-namespace ${c.osc.namespace}`)
    }
    if (c.quic) {
      parts.push(`--quic-port ${c.quic.port}`)
      if (!c.cert) parts.push(`--cert ${c.quic.cert}`)
      if (!c.key) parts.push(`--key ${c.quic.key}`)
    }

    if (c.federation) {
      parts.push(`--federation-hub ${c.federation.hub}`)
      parts.push(`--federation-id ${c.federation.id}`)
      for (const ns of c.federation.namespaces) {
        parts.push(`--federation-namespace ${JSON.stringify(ns)}`)
      }
      if (c.federation.token) parts.push(`--federation-token ${c.federation.token}`)
    }

    if (c.rendezvous) {
      if (c.rendezvous.port) parts.push(`--rendezvous-port ${c.rendezvous.port}`)
      if (c.rendezvous.ttl) parts.push(`--rendezvous-ttl ${c.rendezvous.ttl}`)
    }

    if (c.drainTimeout) parts.push(`--drain-timeout ${c.drainTimeout}`)
    if (c.healthPort) parts.push(`--health-port ${c.healthPort}`)

    return parts.join(' \\\n  ')
  }

  /** Generate Docker Compose YAML. */
  toDockerCompose(options?: { serviceName?: string; image?: string }): string {
    const c = this.config
    const serviceName = options?.serviceName || 'relay'
    const image = options?.image || 'clasp-relay'
    const wsPort = c.port || 7330

    const lines = [
      `  ${serviceName}:`,
      `    image: ${image}`,
      `    command: >`,
      `      ${this.toCommand().replace(/ \\\n  /g, '\n        ')}`,
      `    ports:`,
      `      - "${wsPort}:${wsPort}"`,
    ]

    if (c.authPort) {
      lines.push(`      - "${c.authPort}:${c.authPort}"`)
    }
    if (c.mqtt) {
      lines.push(`      - "${c.mqtt.port}:${c.mqtt.port}"`)
    }
    if (c.osc) {
      lines.push(`      - "${c.osc.port}:${c.osc.port}/udp"`)
    }
    if (c.quic) {
      lines.push(`      - "${c.quic.port}:${c.quic.port}/udp"`)
    }

    // Volumes
    const volumes: string[] = []
    if (c.persist) volumes.push(`      - relay-data:/data`)
    if (typeof c.appConfig !== 'string' && c.appConfig) volumes.push(`      - ./config:/config:ro`)
    if (c.cert) volumes.push(`      - ./certs:/certs:ro`)
    if (c.adminTokenPath) volumes.push(`      - ./secrets:/secrets:ro`)

    if (volumes.length > 0) {
      lines.push(`    volumes:`)
      lines.push(...volumes)
    }

    lines.push(`    restart: unless-stopped`)
    lines.push(`    healthcheck:`)
    lines.push(`      test: ["CMD", "nc", "-z", "localhost", "${wsPort}"]`)
    lines.push(`      interval: 30s`)
    lines.push(`      timeout: 5s`)
    lines.push(`      retries: 3`)

    return lines.join('\n')
  }

  /** Generate environment variable format. */
  toEnv(): string {
    const c = this.config
    const lines: string[] = []

    if (c.port) lines.push(`CLASP_WS_PORT=${c.port}`)
    if (c.host) lines.push(`CLASP_HOST=${c.host}`)
    if (c.name) lines.push(`CLASP_NAME=${c.name}`)
    if (c.authPort) lines.push(`CLASP_AUTH_PORT=${c.authPort}`)
    if (c.corsOrigin) {
      const origins = Array.isArray(c.corsOrigin) ? c.corsOrigin.join(',') : c.corsOrigin
      lines.push(`CLASP_CORS_ORIGIN=${origins}`)
    }
    if (c.maxSessions) lines.push(`CLASP_MAX_SESSIONS=${c.maxSessions}`)
    if (c.sessionTimeout) lines.push(`CLASP_SESSION_TIMEOUT=${c.sessionTimeout}`)
    if (c.paramTtl) lines.push(`CLASP_PARAM_TTL=${c.paramTtl}`)
    if (c.logLevel) {
      lines.push(`RUST_LOG=${c.logLevel}`)
    } else if (c.verbose) {
      lines.push(`RUST_LOG=debug`)
    }

    return lines.join('\n')
  }

  /** Generate app config as JSON string. */
  toAppConfigJSON(): string | null {
    if (!this.config.appConfig || typeof this.config.appConfig === 'string') return null
    return JSON.stringify(this.config.appConfig, null, 2)
  }

  /** Generate rules as JSON string. */
  toRulesJSON(): string | null {
    if (!this.config.rules || typeof this.config.rules === 'string') return null
    return JSON.stringify(this.config.rules, null, 2)
  }

  /** Generate Kubernetes Deployment + Service YAML. */
  toKubernetes(options?: { name?: string; image?: string; namespace?: string }): string {
    const c = this.config
    const name = options?.name || 'clasp-relay'
    const image = options?.image || 'clasp-relay:latest'
    const ns = options?.namespace || 'default'
    const wsPort = c.port || 7330

    const ports: Array<{ name: string; port: number; protocol: string }> = [
      { name: 'ws', port: wsPort, protocol: 'TCP' },
    ]
    if (c.authPort) ports.push({ name: 'auth', port: c.authPort, protocol: 'TCP' })
    if (c.mqtt) ports.push({ name: 'mqtt', port: c.mqtt.port, protocol: 'TCP' })
    if (c.osc) ports.push({ name: 'osc', port: c.osc.port, protocol: 'UDP' })
    if (c.quic) ports.push({ name: 'quic', port: c.quic.port, protocol: 'UDP' })

    const envLines: string[] = []
    if (c.port) envLines.push(`        - name: CLASP_WS_PORT\n          value: "${c.port}"`)
    if (c.authPort) envLines.push(`        - name: CLASP_AUTH_PORT\n          value: "${c.authPort}"`)
    if (c.name) envLines.push(`        - name: CLASP_NAME\n          value: "${c.name}"`)
    if (c.logLevel) envLines.push(`        - name: RUST_LOG\n          value: "${c.logLevel}"`)
    else if (c.verbose) envLines.push(`        - name: RUST_LOG\n          value: "debug"`)

    const containerPorts = ports.map(p =>
      `        - name: ${p.name}\n          containerPort: ${p.port}\n          protocol: ${p.protocol}`
    ).join('\n')

    const servicePorts = ports.map(p =>
      `  - name: ${p.name}\n    port: ${p.port}\n    targetPort: ${p.port}\n    protocol: ${p.protocol}`
    ).join('\n')

    const lines = [
      `apiVersion: apps/v1`,
      `kind: Deployment`,
      `metadata:`,
      `  name: ${name}`,
      `  namespace: ${ns}`,
      `spec:`,
      `  replicas: 1`,
      `  selector:`,
      `    matchLabels:`,
      `      app: ${name}`,
      `  template:`,
      `    metadata:`,
      `      labels:`,
      `        app: ${name}`,
      `    spec:`,
      `      containers:`,
      `      - name: ${name}`,
      `        image: ${image}`,
      `        ports:`,
      containerPorts,
    ]

    if (envLines.length > 0) {
      lines.push(`        env:`)
      lines.push(envLines.join('\n'))
    }

    lines.push(`---`)
    lines.push(`apiVersion: v1`)
    lines.push(`kind: Service`)
    lines.push(`metadata:`)
    lines.push(`  name: ${name}`)
    lines.push(`  namespace: ${ns}`)
    lines.push(`spec:`)
    lines.push(`  selector:`)
    lines.push(`    app: ${name}`)
    lines.push(`  ports:`)
    lines.push(servicePorts)

    return lines.join('\n')
  }

  /** Generate systemd unit file. */
  toSystemd(options?: { user?: string; description?: string }): string {
    const user = options?.user || 'clasp'
    const description = options?.description || 'CLASP Relay Server'
    const cmd = this.toCommand().replace(/ \\\n  /g, ' ')

    const lines = [
      `[Unit]`,
      `Description=${description}`,
      `After=network-online.target`,
      `Wants=network-online.target`,
      ``,
      `[Service]`,
      `Type=simple`,
      `User=${user}`,
      `ExecStart=${cmd}`,
      `Restart=on-failure`,
      `RestartSec=5`,
      `LimitNOFILE=65536`,
      ``,
      `[Install]`,
      `WantedBy=multi-user.target`,
    ]

    return lines.join('\n')
  }
}
