/**
 * Integration tests for @clasp-to/sdk against a real CLASP router.
 *
 * These tests require a running CLASP relay:
 *   clasp-relay --auth-port 7350
 *
 * Set CLASP_TEST_URL to override the default ws://localhost:7330
 * Set CLASP_AUTH_URL to override the default http://localhost:7350
 *
 * Skip with: SKIP_INTEGRATION=1 npx vitest run
 */
import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest'

const ROUTER_URL = process.env.CLASP_TEST_URL || 'ws://localhost:7330'
const AUTH_URL = process.env.CLASP_AUTH_URL || 'http://localhost:7350'
const SKIP = process.env.SKIP_INTEGRATION === '1'

// Dynamic import to avoid loading SDK at module level in skip mode
let clasp: typeof import('../src/easy').default
let EasyClient: typeof import('../src/easy').EasyClient

async function isRouterAvailable(): Promise<boolean> {
  try {
    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), 2000)
    const { ClaspBuilder } = await import('@clasp-to/core')
    const client = await new ClaspBuilder(ROUTER_URL).withReconnect(false).connect()
    client.close()
    clearTimeout(timer)
    return true
  } catch {
    return false
  }
}

async function isAuthAvailable(): Promise<boolean> {
  try {
    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), 2000)
    const res = await fetch(`${AUTH_URL}/auth/guest`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: '{}',
      signal: controller.signal,
    })
    clearTimeout(timer)
    return res.ok || res.status === 401 || res.status === 403
  } catch {
    return false
  }
}

describe.skipIf(SKIP)('Integration: Router Connection', () => {
  let routerAvailable = false
  let clients: any[] = []

  beforeAll(async () => {
    const mod = await import('../src/easy')
    clasp = mod.default
    EasyClient = mod.EasyClient
    routerAvailable = await isRouterAvailable()
  })

  afterEach(() => {
    for (const c of clients) {
      try { c.close() } catch {}
    }
    clients = []
  })

  it.skipIf(!routerAvailable)('connects to router anonymously', async () => {
    const c = await clasp(ROUTER_URL)
    clients.push(c)
    expect(c.connected).toBe(true)
    expect(c.session).toBeTruthy()
  })

  it.skipIf(!routerAvailable)('connects with a name', async () => {
    const c = await clasp(ROUTER_URL, { name: 'Integration Test' })
    clients.push(c)
    expect(c.connected).toBe(true)
  })

  it.skipIf(!routerAvailable)('set and get round-trip', async () => {
    const c = await clasp(ROUTER_URL, { name: 'SetGet Test' })
    clients.push(c)

    const address = `/test/sdk/setget/${Date.now()}`
    await c.set(address, 42)
    await new Promise(r => setTimeout(r, 100))

    const val = await c.get(address)
    expect(val).toBe(42)
  })

  it.skipIf(!routerAvailable)('subscribe receives set values', async () => {
    const c = await clasp(ROUTER_URL, { name: 'Sub Test' })
    clients.push(c)

    const address = `/test/sdk/sub/${Date.now()}`
    const received: any[] = []

    c.on(address, (value, addr) => {
      received.push({ value, addr })
    })

    await new Promise(r => setTimeout(r, 50))

    await c.set(address, 'hello')
    await new Promise(r => setTimeout(r, 100))

    expect(received.length).toBeGreaterThanOrEqual(1)
    expect(received[0].value).toBe('hello')
    expect(received[0].addr).toBe(address)
  })

  it.skipIf(!routerAvailable)('wildcard subscription works', async () => {
    const c = await clasp(ROUTER_URL, { name: 'Wildcard Test' })
    clients.push(c)

    const base = `/test/sdk/wild/${Date.now()}`
    const received: string[] = []

    c.on(`${base}/**`, (value, addr) => {
      received.push(addr)
    })

    await new Promise(r => setTimeout(r, 50))

    await c.set(`${base}/a`, 1)
    await c.set(`${base}/b`, 2)
    await c.set(`${base}/c/d`, 3)
    await new Promise(r => setTimeout(r, 150))

    expect(received.length).toBeGreaterThanOrEqual(3)
    expect(received).toContain(`${base}/a`)
    expect(received).toContain(`${base}/b`)
    expect(received).toContain(`${base}/c/d`)
  })

  it.skipIf(!routerAvailable)('emit delivers events', async () => {
    const c = await clasp(ROUTER_URL, { name: 'Emit Test' })
    clients.push(c)

    const address = `/test/sdk/emit/${Date.now()}`
    const received: any[] = []

    c.on(address, (value) => {
      received.push(value)
    })

    await new Promise(r => setTimeout(r, 50))

    await c.emit(address, { event: 'fired' })
    await new Promise(r => setTimeout(r, 100))

    expect(received.length).toBeGreaterThanOrEqual(1)
  })

  it.skipIf(!routerAvailable)('stream delivers high-rate data', async () => {
    const c = await clasp(ROUTER_URL, { name: 'Stream Test' })
    clients.push(c)

    const address = `/test/sdk/stream/${Date.now()}`
    const received: any[] = []

    c.on(address, (value) => {
      received.push(value)
    })

    await new Promise(r => setTimeout(r, 50))

    for (let i = 0; i < 5; i++) {
      c.stream(address, { sample: i })
    }
    await new Promise(r => setTimeout(r, 200))

    expect(received.length).toBeGreaterThanOrEqual(1)
  })

  it.skipIf(!routerAvailable)('bundle delivers atomically', async () => {
    const c = await clasp(ROUTER_URL, { name: 'Bundle Test' })
    clients.push(c)

    const base = `/test/sdk/bundle/${Date.now()}`
    const received: string[] = []

    c.on(`${base}/**`, (value, addr) => {
      received.push(addr)
    })

    await new Promise(r => setTimeout(r, 50))

    c.bundle([
      { set: [`${base}/a`, 1] },
      { set: [`${base}/b`, 2] },
    ])
    await new Promise(r => setTimeout(r, 150))

    expect(received).toContain(`${base}/a`)
    expect(received).toContain(`${base}/b`)
  })

  it.skipIf(!routerAvailable)('time returns server time', async () => {
    const c = await clasp(ROUTER_URL, { name: 'Time Test' })
    clients.push(c)

    const t = c.time()
    const nowUs = Date.now() * 1000
    expect(Math.abs(t - nowUs)).toBeLessThan(10_000_000)
  })

  it.skipIf(!routerAvailable)('two clients communicate', async () => {
    const c1 = await clasp(ROUTER_URL, { name: 'Sender' })
    const c2 = await clasp(ROUTER_URL, { name: 'Receiver' })
    clients.push(c1, c2)

    const address = `/test/sdk/multi/${Date.now()}`
    const received: any[] = []

    c2.on(address, (value) => {
      received.push(value)
    })

    await new Promise(r => setTimeout(r, 50))

    await c1.set(address, 'from-c1')
    await new Promise(r => setTimeout(r, 150))

    expect(received).toContain('from-c1')
  })

  it.skipIf(!routerAvailable)('cached returns last set value', async () => {
    const c = await clasp(ROUTER_URL, { name: 'Cache Test' })
    clients.push(c)

    const address = `/test/sdk/cache/${Date.now()}`

    c.on(address, () => {})
    await new Promise(r => setTimeout(r, 50))

    await c.set(address, 'cached-value')
    await new Promise(r => setTimeout(r, 100))

    expect(c.cached(address)).toBe('cached-value')
  })

  it.skipIf(!routerAvailable)('unsubscribe stops delivery', async () => {
    const c = await clasp(ROUTER_URL, { name: 'Unsub Test' })
    clients.push(c)

    const address = `/test/sdk/unsub/${Date.now()}`
    const received: any[] = []

    const unsub = c.on(address, (value) => {
      received.push(value)
    })

    await new Promise(r => setTimeout(r, 50))

    await c.set(address, 'before')
    await new Promise(r => setTimeout(r, 100))

    unsub()
    await new Promise(r => setTimeout(r, 50))

    await c.set(address, 'after')
    await new Promise(r => setTimeout(r, 100))

    expect(received).toContain('before')
  })

  it.skipIf(!routerAvailable)('close disconnects cleanly', async () => {
    const c = await clasp(ROUTER_URL, { name: 'Close Test' })
    expect(c.connected).toBe(true)
    c.close()
    expect(c.connected).toBe(false)
  })

  it.skipIf(!routerAvailable)('multiple subscribers on same address all receive', async () => {
    const c = await clasp(ROUTER_URL, { name: 'MultiSub Test' })
    clients.push(c)

    const address = `/test/sdk/multisub/${Date.now()}`
    const r1: any[] = []
    const r2: any[] = []

    c.on(address, (v) => r1.push(v))
    c.on(address, (v) => r2.push(v))

    await new Promise(r => setTimeout(r, 50))

    await c.set(address, 'both')
    await new Promise(r => setTimeout(r, 100))

    expect(r1).toContain('both')
    expect(r2).toContain('both')
  })

  it.skipIf(!routerAvailable)('set with complex nested object round-trips', async () => {
    const c = await clasp(ROUTER_URL, { name: 'Complex Test' })
    clients.push(c)

    const address = `/test/sdk/complex/${Date.now()}`
    const complex = { a: [1, 2, { b: true }], c: 'hello', d: null }

    await c.set(address, complex)
    await new Promise(r => setTimeout(r, 100))

    const val = await c.get(address)
    expect(val).toEqual(complex)
  })

  it.skipIf(!routerAvailable)('subscribe() alias works', async () => {
    const c = await clasp(ROUTER_URL, { name: 'Subscribe Test' })
    clients.push(c)

    const address = `/test/sdk/subscribe/${Date.now()}`
    const received: any[] = []

    c.subscribe(address, (value) => {
      received.push(value)
    })

    await new Promise(r => setTimeout(r, 50))

    await c.set(address, 'via-subscribe')
    await new Promise(r => setTimeout(r, 100))

    expect(received).toContain('via-subscribe')
  })
})

describe.skipIf(SKIP)('Integration: Auth Server', () => {
  let authAvailable = false
  let clients: any[] = []

  beforeAll(async () => {
    const mod = await import('../src/easy')
    clasp = mod.default
    authAvailable = await isAuthAvailable()
  })

  afterEach(() => {
    for (const c of clients) {
      try { c.close() } catch {}
    }
    clients = []
  })

  it.skipIf(!authAvailable)('guest access returns a device', async () => {
    const c = await clasp(ROUTER_URL, { authUrl: AUTH_URL })
    clients.push(c)

    const guest = await c.guest({ scopes: ['read:/**'] })
    expect(guest.token).toBeTruthy()
    expect(guest.token.startsWith('cpsk_')).toBe(true)
  })

  it.skipIf(!authAvailable)('register creates a device', async () => {
    const c = await clasp(ROUTER_URL, { authUrl: AUTH_URL })
    clients.push(c)

    const username = `test-${Date.now()}`
    const device = await c.register({
      name: 'Test Device',
      username,
      password: 'test-password-123',
      scopes: ['read:/**', 'write:/test/**'],
    })

    expect(device.token).toBeTruthy()
    expect(device.name).toBe('Test Device')
  })

  it.skipIf(!authAvailable)('login returns device with token', async () => {
    const c = await clasp(ROUTER_URL, { authUrl: AUTH_URL })
    clients.push(c)

    const username = `login-test-${Date.now()}`

    await c.register({
      name: 'Login Test',
      username,
      password: 'test-password-123',
    })

    const device = await c.login({
      username,
      password: 'test-password-123',
    })

    expect(device.token).toBeTruthy()
    expect(device.token.startsWith('cpsk_')).toBe(true)
  })

  it.skipIf(!authAvailable)('device can connect with token', async () => {
    const c = await clasp(ROUTER_URL, { authUrl: AUTH_URL })
    clients.push(c)

    const guest = await c.guest({ scopes: ['read:/**', 'write:/test/**'] })
    const conn = await guest.connect()
    clients.push(conn)

    expect(conn.connected).toBe(true)

    await conn.set(`/test/sdk/auth/${Date.now()}`, 'authenticated')
  })

  it.skipIf(!authAvailable)('provision returns working CredentialBundle', async () => {
    const c = await clasp(ROUTER_URL, { authUrl: AUTH_URL })
    clients.push(c)

    const parent = await c.register({
      name: `prov-parent-${Date.now()}`,
      scopes: ['read:/**', 'write:/test/**'],
    })

    const creds = await parent.provision({
      name: 'Provisioned',
      scopes: ['read:/**'],
    })

    expect(creds.token).toBeTruthy()
    const json = JSON.parse(creds.toJSON())
    expect(json.token).toBe(creds.token)

    const env = creds.toEnv()
    expect(env).toContain('CLASP_TOKEN=')
  })
})
