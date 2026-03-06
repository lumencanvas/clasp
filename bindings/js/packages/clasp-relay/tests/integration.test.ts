import { describe, it, expect, afterEach } from 'vitest'
import { createRelay, RelayServer, resolveBinary } from '../src/index'
import { request } from 'http'
import { join } from 'path'
import WebSocket from 'ws'

const OPTS = { readyTimeout: 15_000 }

/** Resolve the relay binary from the repo build output or PATH. */
const RELAY_BIN = (() => {
  // tests/ -> clasp-relay/ -> packages/ -> js/ -> bindings/ -> repo root
  const repoRoot = join(__dirname, '..', '..', '..', '..', '..')
  const candidates = [
    join(repoRoot, 'deploy', 'relay', 'target', 'release', 'clasp-relay'),
    join(repoRoot, 'deploy', 'relay', 'target', 'debug', 'clasp-relay'),
  ]
  for (const candidate of candidates) {
    try { return resolveBinary(candidate) } catch { /* try next */ }
  }
  try { return resolveBinary() } catch { return null }
})()

const describeIntegration = RELAY_BIN ? describe : describe.skip

// Track all servers for cleanup
const servers: RelayServer[] = []

afterEach(async () => {
  await Promise.all(servers.map(s => s.stopped ? Promise.resolve() : s.stop(3000)))
  servers.length = 0
})

// --- Helpers ---

function httpGet(url: string): Promise<{ status: number; body: string }> {
  return new Promise((resolve, reject) => {
    const parsed = new URL(url)
    const req = request(
      { host: parsed.hostname, port: parsed.port, path: parsed.pathname, method: 'GET', timeout: 2000 },
      (res) => {
        let body = ''
        res.on('data', (chunk: Buffer) => { body += chunk.toString() })
        res.on('end', () => resolve({ status: res.statusCode!, body }))
      },
    )
    req.on('error', reject)
    req.on('timeout', () => { req.destroy(); reject(new Error('HTTP timeout')) })
    req.end()
  })
}

function httpPost(url: string, data: object): Promise<{ status: number; body: string }> {
  return new Promise((resolve, reject) => {
    const parsed = new URL(url)
    const payload = JSON.stringify(data)
    const req = request(
      {
        host: parsed.hostname, port: parsed.port, path: parsed.pathname,
        method: 'POST', timeout: 2000,
        headers: { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(payload) },
      },
      (res) => {
        let body = ''
        res.on('data', (chunk: Buffer) => { body += chunk.toString() })
        res.on('end', () => resolve({ status: res.statusCode!, body }))
      },
    )
    req.on('error', reject)
    req.on('timeout', () => { req.destroy(); reject(new Error('HTTP timeout')) })
    req.write(payload)
    req.end()
  })
}

function openWs(url: string): Promise<WebSocket> {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url)
    const timer = setTimeout(() => { ws.close(); reject(new Error('WS connect timeout')) }, 5000)
    ws.on('open', () => { clearTimeout(timer); resolve(ws) })
    ws.on('error', (err) => { clearTimeout(timer); reject(err) })
  })
}

// Port allocator to avoid collisions
let nextPort = 18300
function allocPorts(count: number): number[] {
  const ports = []
  for (let i = 0; i < count; i++) ports.push(nextPort++)
  return ports
}

// Unique ID for usernames
let uid = 0
function uniqueName(prefix: string): string {
  return `${prefix}_${Date.now()}_${uid++}`
}

function startRelay(config: Parameters<typeof createRelay>[0]) {
  return createRelay(config, { binary: RELAY_BIN!, ...OPTS })
}

// --- Tests ---

describeIntegration('relay: start and stop', () => {
  it('starts with config object and stops gracefully', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay({ port: wsPort, drainTimeout: 1 })
    servers.push(server)

    expect(server.pid).toBeGreaterThan(0)
    expect(server.url).toBe(`ws://localhost:${wsPort}`)
    expect(server.stopped).toBe(false)
    expect(server.exitCode).toBeNull()

    await server.stop()
    expect(server.stopped).toBe(true)
  }, 20_000)

  it('starts with builder callback', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay(r => r.port(wsPort).name('Builder Test').drainTimeout(1))
    servers.push(server)

    expect(server.url).toBe(`ws://localhost:${wsPort}`)
    await server.stop()
  }, 20_000)

  it('force-kills with kill()', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay({ port: wsPort, drainTimeout: 1 })
    servers.push(server)

    const exitPromise = new Promise<void>(resolve => {
      if (server.stopped) return resolve()
      server.on('exit', () => resolve())
    })

    server.kill()
    await exitPromise
    expect(server.stopped).toBe(true)
  }, 20_000)

  it('emits exit event on stop', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay({ port: wsPort, drainTimeout: 1 })
    servers.push(server)

    const exitPromise = new Promise<[number | null, string | null]>(resolve => {
      server.on('exit', (code, signal) => resolve([code, signal]))
    })

    await server.stop()
    const [code, signal] = await exitPromise
    expect(code === 0 || signal === 'SIGTERM' || signal === 'SIGKILL').toBe(true)
  }, 20_000)
})

describeIntegration('relay: log capture', () => {
  it('captures startup logs including banner', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay({ port: wsPort, verbose: true, drainTimeout: 1 })
    servers.push(server)

    expect(server.logs.length).toBeGreaterThan(0)
    const allLogs = server.logs.join('\n')
    expect(allLogs).toMatch(/CLASP|accepting connections|WebSocket/)

    await server.stop()
  }, 20_000)

  it('emits log events for connection activity', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay({ port: wsPort, verbose: true, drainTimeout: 1 })
    servers.push(server)

    const logLines: string[] = []
    server.on('log', (line: string) => logLines.push(line))

    // Connect and disconnect to generate log activity
    const ws = await openWs(`ws://localhost:${wsPort}`)
    ws.close()
    await new Promise(resolve => setTimeout(resolve, 500))

    expect(logLines.length).toBeGreaterThan(0)

    await server.stop()
  }, 20_000)
})

describeIntegration('relay: WebSocket connectivity', () => {
  it('accepts a WebSocket connection on the configured port', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay({ port: wsPort, drainTimeout: 1 })
    servers.push(server)

    const ws = await openWs(`ws://localhost:${wsPort}`)
    expect(ws.readyState).toBe(WebSocket.OPEN)
    ws.close()

    await server.stop()
  }, 20_000)

  it('accepts multiple concurrent WebSocket connections', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay({ port: wsPort, maxSessions: 10, drainTimeout: 1 })
    servers.push(server)

    const sockets = await Promise.all([
      openWs(`ws://localhost:${wsPort}`),
      openWs(`ws://localhost:${wsPort}`),
      openWs(`ws://localhost:${wsPort}`),
    ])

    for (const ws of sockets) {
      expect(ws.readyState).toBe(WebSocket.OPEN)
      ws.close()
    }

    await server.stop()
  }, 20_000)

  it('refuses connections after stop', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay({ port: wsPort, drainTimeout: 1 })
    servers.push(server)

    await server.stop()

    await expect(openWs(`ws://localhost:${wsPort}`)).rejects.toThrow()
  }, 20_000)
})

describeIntegration('relay: health check endpoint', () => {
  it('/healthz returns 200 ok', async () => {
    const [wsPort, healthPort] = allocPorts(2)
    const server = await startRelay({ port: wsPort, healthPort, drainTimeout: 1 })
    servers.push(server)

    const res = await httpGet(`http://localhost:${healthPort}/healthz`)
    expect(res.status).toBe(200)
    expect(res.body.trim()).toBe('ok')

    await server.stop()
  }, 20_000)

  it('/readyz returns 200 ready', async () => {
    const [wsPort, healthPort] = allocPorts(2)
    const server = await startRelay({ port: wsPort, healthPort, drainTimeout: 1 })
    servers.push(server)

    const res = await httpGet(`http://localhost:${healthPort}/readyz`)
    expect(res.status).toBe(200)
    expect(res.body.trim()).toBe('ready')

    await server.stop()
  }, 20_000)

  it('sets healthUrl property', async () => {
    const [wsPort, healthPort] = allocPorts(2)
    const server = await startRelay({ port: wsPort, healthPort, drainTimeout: 1 })
    servers.push(server)

    expect(server.healthUrl).toBe(`http://localhost:${healthPort}`)

    await server.stop()
  }, 20_000)
})

describeIntegration('relay: auth API', () => {
  it('registers a user and returns a token', async () => {
    const [wsPort, authPort] = allocPorts(2)
    const server = await startRelay({ port: wsPort, authPort, drainTimeout: 1 })
    servers.push(server)

    const username = uniqueName('reg')
    const res = await httpPost(`http://localhost:${authPort}/auth/register`, {
      username,
      password: 'testpass123',
    })

    expect(res.status).toBe(200)
    const body = JSON.parse(res.body)
    expect(body.token).toBeTruthy()
    expect(typeof body.token).toBe('string')
    expect(body.username).toBe(username)
    expect(body.user_id).toBeTruthy()

    await server.stop()
  }, 20_000)

  it('logs in after registration', async () => {
    const [wsPort, authPort] = allocPorts(2)
    const server = await startRelay({ port: wsPort, authPort, drainTimeout: 1 })
    servers.push(server)

    const username = uniqueName('login')
    const password = 'password456'

    // Register
    const regRes = await httpPost(`http://localhost:${authPort}/auth/register`, { username, password })
    expect(regRes.status).toBe(200)

    // Login
    const loginRes = await httpPost(`http://localhost:${authPort}/auth/login`, { username, password })
    expect(loginRes.status).toBe(200)
    const body = JSON.parse(loginRes.body)
    expect(body.token).toBeTruthy()
    expect(body.username).toBe(username)

    await server.stop()
  }, 20_000)

  it('issues guest tokens', async () => {
    const [wsPort, authPort] = allocPorts(2)
    const server = await startRelay({ port: wsPort, authPort, drainTimeout: 1 })
    servers.push(server)

    const res = await httpPost(`http://localhost:${authPort}/auth/guest`, {
      name: 'Guest User',
    })

    expect(res.status).toBe(200)
    const body = JSON.parse(res.body)
    expect(body.token).toBeTruthy()
    expect(body.username).toBe('Guest User')

    await server.stop()
  }, 20_000)

  it('rejects login with wrong password', async () => {
    const [wsPort, authPort] = allocPorts(2)
    const server = await startRelay({ port: wsPort, authPort, drainTimeout: 1 })
    servers.push(server)

    const username = uniqueName('wrongpw')
    await httpPost(`http://localhost:${authPort}/auth/register`, { username, password: 'correct' })

    const res = await httpPost(`http://localhost:${authPort}/auth/login`, { username, password: 'wrong' })
    expect(res.status).toBe(401)

    await server.stop()
  }, 20_000)
})

describeIntegration('relay: multiple servers', () => {
  it('runs two relays on different ports with independent connectivity', async () => {
    const [wsPort1, wsPort2] = allocPorts(2)
    const s1 = await startRelay({ port: wsPort1, name: 'Relay A', drainTimeout: 1 })
    const s2 = await startRelay({ port: wsPort2, name: 'Relay B', drainTimeout: 1 })
    servers.push(s1, s2)

    expect(s1.pid).not.toBe(s2.pid)

    // Both accept connections
    const ws1 = await openWs(`ws://localhost:${wsPort1}`)
    const ws2 = await openWs(`ws://localhost:${wsPort2}`)
    expect(ws1.readyState).toBe(WebSocket.OPEN)
    expect(ws2.readyState).toBe(WebSocket.OPEN)
    ws1.close()
    ws2.close()

    // Stop one, other stays up
    await s1.stop()
    expect(s1.stopped).toBe(true)
    expect(s2.stopped).toBe(false)

    const ws2Again = await openWs(`ws://localhost:${wsPort2}`)
    expect(ws2Again.readyState).toBe(WebSocket.OPEN)
    ws2Again.close()

    await s2.stop()
  }, 30_000)
})

describeIntegration('relay: inline appConfig', () => {
  it('starts with inline appConfig object (written to temp file)', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay({
      port: wsPort,
      drainTimeout: 1,
      appConfig: { scopes: ['read:/**', 'write:/**'] },
    })
    servers.push(server)

    expect(server.stopped).toBe(false)
    const ws = await openWs(`ws://localhost:${wsPort}`)
    expect(ws.readyState).toBe(WebSocket.OPEN)
    ws.close()

    await server.stop()
  }, 20_000)
})

describeIntegration('relay: configuration', () => {
  it('applies custom drain timeout (fast shutdown)', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay({ port: wsPort, drainTimeout: 1 })
    servers.push(server)

    const start = Date.now()
    await server.stop(10_000)
    const elapsed = Date.now() - start

    // drainTimeout=1s means the relay exits in ~1s after SIGTERM
    expect(elapsed).toBeLessThan(5000)
  }, 20_000)

  it('verbose mode produces debug-level logs', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay(r => r.port(wsPort).verbose().drainTimeout(1))
    servers.push(server)

    const allLogs = server.logs.join('\n')
    expect(allLogs).toMatch(/DEBUG|TRACE|debug|trace/)

    await server.stop()
  }, 20_000)
})

describeIntegration('relay: WS-only readiness (no auth/health port)', () => {
  it('detects readiness via TCP probe when no HTTP ports configured', async () => {
    const [wsPort] = allocPorts(1)
    const server = await startRelay({ port: wsPort, drainTimeout: 1 })
    servers.push(server)

    // Verify actually ready
    const ws = await openWs(`ws://localhost:${wsPort}`)
    expect(ws.readyState).toBe(WebSocket.OPEN)
    ws.close()

    await server.stop()
  }, 20_000)
})
