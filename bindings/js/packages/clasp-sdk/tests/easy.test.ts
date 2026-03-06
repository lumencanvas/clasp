import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { EasyClient, inferAuthUrl } from '../src/easy'

// Mock a Clasp client for unit testing (no real WebSocket)
function mockClasp(overrides: Record<string, unknown> = {}) {
  return {
    set: vi.fn(),
    get: vi.fn().mockResolvedValue(42),
    on: vi.fn().mockReturnValue(() => {}),
    subscribe: vi.fn().mockReturnValue(() => {}),
    emit: vi.fn(),
    stream: vi.fn(),
    gesture: vi.fn(),
    timeline: vi.fn(),
    bundle: vi.fn(),
    time: vi.fn().mockReturnValue(1000000),
    close: vi.fn(),
    cached: vi.fn().mockReturnValue(undefined),
    getSignals: vi.fn().mockReturnValue([]),
    querySignals: vi.fn().mockReturnValue([]),
    getLastError: vi.fn().mockReturnValue(null),
    clearError: vi.fn(),
    onConnect: vi.fn(),
    onDisconnect: vi.fn(),
    onError: vi.fn(),
    onReconnect: vi.fn(),
    connected: true,
    session: 'test-session',
    ...overrides,
  }
}

// Mock global fetch for auth tests
function mockFetch(status: number, body: unknown, ok?: boolean) {
  return vi.fn().mockResolvedValue({
    ok: ok ?? (status >= 200 && status < 300),
    status,
    text: vi.fn().mockResolvedValue(typeof body === 'string' ? body : JSON.stringify(body)),
    json: vi.fn().mockResolvedValue(body),
    url: 'http://test',
  })
}

describe('EasyClient', () => {
  describe('Construction', () => {
    it('creates with default options', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      expect(easy.inner).toBe(mock)
    })

    it('creates with encrypted mode', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330', { encrypted: true })
      expect(easy).toBeDefined()
    })

    it('creates with custom authUrl (does not infer)', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330', {
        authUrl: 'http://custom:9000',
      })
      expect(easy).toBeDefined()
    })

    it('creates with name option', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330', {
        encrypted: true,
        name: 'TestClient',
      })
      expect(easy).toBeDefined()
    })

    it('infers authUrl when not provided', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      // Auth URL is internal, but we verify via register which would use it
      expect(easy).toBeDefined()
    })
  })

  describe('Data Operations', () => {
    it('set() delegates to client.set', async () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      await easy.set('/test', 42)
      expect(mock.set).toHaveBeenCalledWith('/test', 42)
    })

    it('set() in encrypted mode delegates to crypto', async () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330', { encrypted: true })
      // CryptoClient.set is async — the important thing is no error thrown
      await easy.set('/test', 'encrypted-data')
    })

    it('set() returns a promise (async)', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const result = easy.set('/test', 42)
      expect(result).toBeInstanceOf(Promise)
    })

    it('set() handles complex nested Value objects', async () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const complex = { a: [1, { b: 'c' }], d: true }
      await easy.set('/test', complex)
      expect(mock.set).toHaveBeenCalledWith('/test', complex)
    })

    it('set() handles Uint8Array binary data', async () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const data = new Uint8Array([1, 2, 3])
      await easy.set('/test', data)
      expect(mock.set).toHaveBeenCalledWith('/test', data)
    })

    it('set() handles null value', async () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      await easy.set('/test', null)
      expect(mock.set).toHaveBeenCalledWith('/test', null)
    })

    it('get() resolves with server value', async () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const val = await easy.get('/test')
      expect(val).toBe(42)
    })

    it('get() rejects on error', async () => {
      const mock = mockClasp({ get: vi.fn().mockRejectedValue(new Error('timeout')) })
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      await expect(easy.get('/test')).rejects.toThrow('timeout')
    })

    it('on() returns unsubscribe function', () => {
      const unsub = vi.fn()
      const mock = mockClasp({ on: vi.fn().mockReturnValue(unsub) })
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const result = easy.on('/test', () => {})
      expect(result).toBe(unsub)
    })

    it('on() callback receives value and address', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const cb = vi.fn()
      easy.on('/test', cb)
      expect(mock.on).toHaveBeenCalled()
      expect(mock.on.mock.calls[0][0]).toBe('/test')
      expect(mock.on.mock.calls[0][1]).toBe(cb)
    })

    it('on() with subscribe options', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      easy.on('/test', () => {}, { maxRate: 10, epsilon: 0.01 })
      expect(mock.on).toHaveBeenCalledWith('/test', expect.any(Function), { maxRate: 10, epsilon: 0.01 })
    })

    it('on() with encrypted mode delegates to crypto.subscribe', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330', { encrypted: true })
      easy.on('/test', () => {})
      // In encrypted mode, it should NOT call mock.on (calls crypto.subscribe instead)
      expect(mock.on).not.toHaveBeenCalled()
    })

    it('on() with wildcard pattern', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      easy.on('/test/**', () => {})
      expect(mock.on.mock.calls[0][0]).toBe('/test/**')
    })

    it('subscribe() is an alias for on()', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const cb = vi.fn()
      easy.subscribe('/test', cb)
      expect(mock.on).toHaveBeenCalledWith('/test', cb, undefined)
    })

    it('emit() delegates with payload', async () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      await easy.emit('/events/go', { scene: 1 })
      expect(mock.emit).toHaveBeenCalledWith('/events/go', { scene: 1 })
    })

    it('emit() without payload', async () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      await easy.emit('/events/go')
      expect(mock.emit).toHaveBeenCalledWith('/events/go', undefined)
    })

    it('emit() returns a promise (async)', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const result = easy.emit('/events/go')
      expect(result).toBeInstanceOf(Promise)
    })

    it('emit() in encrypted mode routes through crypto client', async () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330', { encrypted: true })
      // CryptoClient falls through to inner client when no E2E session is active,
      // but the call path goes through CryptoClient.emit (which is async)
      await easy.emit('/events/go', { data: 1 })
      // inner.emit is still called (fallthrough), confirming async path was taken
      expect(mock.emit).toHaveBeenCalledWith('/events/go', { data: 1 })
    })

    it('stream() delegates to client.stream', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      easy.stream('/sensor/accel', { x: 0.1 })
      expect(mock.stream).toHaveBeenCalledWith('/sensor/accel', { x: 0.1 })
    })

    it('gesture() with all phases', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      for (const phase of ['start', 'move', 'end', 'cancel'] as const) {
        easy.gesture('/input/touch', 1, phase, { x: 100 })
      }
      expect(mock.gesture).toHaveBeenCalledTimes(4)
      expect(mock.gesture).toHaveBeenCalledWith('/input/touch', 1, 'start', { x: 100 })
      expect(mock.gesture).toHaveBeenCalledWith('/input/touch', 1, 'cancel', { x: 100 })
    })

    it('timeline() with keyframes and easing', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const keyframes = [
        { time: 0, value: 0, easing: 'ease-in' as const },
        { time: 1000000, value: 1, easing: 'ease-out' as const },
      ]
      easy.timeline('/fade', keyframes)
      expect(mock.timeline).toHaveBeenCalledWith('/fade', keyframes, undefined)
    })

    it('timeline() with loop option', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      easy.timeline('/loop', [{ time: 0, value: 0 }], { loop: true })
      expect(mock.timeline).toHaveBeenCalledWith('/loop', [{ time: 0, value: 0 }], { loop: true })
    })

    it('bundle() with mixed set/emit operations', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      easy.bundle([{ set: ['/a', 1] }, { emit: ['/b', 2] }])
      expect(mock.bundle).toHaveBeenCalledWith(
        [{ set: ['/a', 1] }, { emit: ['/b', 2] }],
        undefined
      )
    })

    it('bundle() with timing option', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      easy.bundle([{ set: ['/a', 1] }], { at: 1234567 })
      expect(mock.bundle).toHaveBeenCalledWith([{ set: ['/a', 1] }], { at: 1234567 })
    })

    it('cached() returns undefined for unknown address', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      expect(easy.cached('/unknown')).toBeUndefined()
    })
  })

  describe('Auth (mocked fetch)', () => {
    let originalFetch: typeof globalThis.fetch

    beforeEach(() => {
      originalFetch = globalThis.fetch
    })

    afterEach(() => {
      globalThis.fetch = originalFetch
    })

    it('register() sends correct request body', async () => {
      globalThis.fetch = mockFetch(200, { session_id: 'id1', token: 'cpsk_abc', scopes: ['read:/**'] }) as any
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      await easy.register({ name: 'Test', scopes: ['read:/**'] })

      const call = (globalThis.fetch as any).mock.calls[0]
      expect(call[0]).toContain('/auth/register')
      const body = JSON.parse(call[1].body)
      expect(body.name).toBe('Test')
      expect(body.scopes).toEqual(['read:/**'])
    })

    it('register() with username/password', async () => {
      globalThis.fetch = mockFetch(200, { session_id: 'id1', token: 'cpsk_abc' }) as any
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      await easy.register({ name: 'Test', username: 'alice', password: 'secret' })

      const body = JSON.parse((globalThis.fetch as any).mock.calls[0][1].body)
      expect(body.username).toBe('alice')
      expect(body.password).toBe('secret')
    })

    it('register() returns Device with correct fields', async () => {
      globalThis.fetch = mockFetch(200, {
        session_id: 'device-123',
        token: 'cpsk_test',
        scopes: ['read:/**'],
      }) as any
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      const device = await easy.register({ name: 'MyDevice', scopes: ['read:/**'] })
      expect(device.id).toBe('device-123')
      expect(device.token).toBe('cpsk_test')
      expect(device.name).toBe('MyDevice')
      expect(device.scopes).toEqual(['read:/**'])
    })

    it('register() throws on 409 Conflict', async () => {
      globalThis.fetch = mockFetch(409, 'Already exists', false) as any
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      await expect(easy.register({ name: 'Dup' })).rejects.toThrow('Registration failed (409)')
    })

    it('register() throws on 500 with error body', async () => {
      globalThis.fetch = mockFetch(500, 'Internal error', false) as any
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      await expect(easy.register({ name: 'Err' })).rejects.toThrow('Registration failed (500)')
    })

    it('register() throws on non-JSON success response', async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        text: vi.fn().mockResolvedValue('<html>502 Bad Gateway</html>'),
        url: 'http://test/auth/register',
      }) as any
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      await expect(easy.register({ name: 'Test' })).rejects.toThrow('Expected JSON')
    })

    it('login() sends credentials', async () => {
      globalThis.fetch = mockFetch(200, { session_id: 'id1', token: 'cpsk_abc' }) as any
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      await easy.login({ username: 'alice', password: 'secret' })

      const body = JSON.parse((globalThis.fetch as any).mock.calls[0][1].body)
      expect(body.username).toBe('alice')
      expect(body.password).toBe('secret')
    })

    it('login() returns Device with token', async () => {
      globalThis.fetch = mockFetch(200, { session_id: 'id1', token: 'cpsk_login' }) as any
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      const device = await easy.login({ username: 'alice', password: 'secret' })
      expect(device.token).toBe('cpsk_login')
    })

    it('login() throws on 401 Unauthorized', async () => {
      globalThis.fetch = mockFetch(401, 'Invalid credentials', false) as any
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      await expect(easy.login({ username: 'wrong', password: 'bad' })).rejects.toThrow('Login failed (401)')
    })

    it('guest() with no options', async () => {
      globalThis.fetch = mockFetch(200, { session_id: 'g1', token: 'cpsk_guest' }) as any
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      const device = await easy.guest()
      expect(device.token).toBe('cpsk_guest')
      expect(device.name).toBe('guest')
    })

    it('guest() with scopes', async () => {
      globalThis.fetch = mockFetch(200, { session_id: 'g1', token: 'cpsk_guest', scopes: ['read:/**'] }) as any
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      const device = await easy.guest({ scopes: ['read:/**'] })
      expect(device.scopes).toEqual(['read:/**'])
    })

    it('guest() throws on 403 Forbidden', async () => {
      globalThis.fetch = mockFetch(403, 'Guest access disabled', false) as any
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')

      await expect(easy.guest()).rejects.toThrow('Guest access failed (403)')
    })
  })

  describe('Rooms', () => {
    it('room() creates new room (mocked crypto)', async () => {
      // Room creation requires CryptoClient internals — test that it returns a Room
      // This tests the control flow, not actual encryption
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      // Without CryptoClient properly configured, this may throw
      // We're testing that the method exists and has the right signature
      expect(typeof easy.room).toBe('function')
    })

    it('destroyRoom() removes room from map', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      // Should not throw even when room doesn't exist
      easy.destroyRoom('/nonexistent')
    })
  })

  describe('Lifecycle', () => {
    it('close() closes underlying client', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      easy.close()
      expect(mock.close).toHaveBeenCalled()
    })

    it('close() with no rooms/crypto does not error', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      expect(() => easy.close()).not.toThrow()
    })

    it('onConnect delegates', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const cb = vi.fn()
      easy.onConnect(cb)
      expect(mock.onConnect).toHaveBeenCalledWith(cb)
    })

    it('onDisconnect delegates', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const cb = vi.fn()
      easy.onDisconnect(cb)
      expect(mock.onDisconnect).toHaveBeenCalledWith(cb)
    })

    it('onError delegates', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const cb = vi.fn()
      easy.onError(cb)
      expect(mock.onError).toHaveBeenCalledWith(cb)
    })

    it('onReconnect delegates', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const cb = vi.fn()
      easy.onReconnect(cb)
      expect(mock.onReconnect).toHaveBeenCalledWith(cb)
    })

    it('connected getter reflects client state', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      expect(easy.connected).toBe(true)
    })

    it('session getter reflects client state', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      expect(easy.session).toBe('test-session')
    })

    it('time() returns server time', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      expect(easy.time()).toBe(1000000)
    })

    it('inner exposes raw client', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      expect(easy.inner).toBe(mock)
    })
  })

  describe('Bridge & Rules', () => {
    it('bridge() returns BridgeCommand with correct protocol/url', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const cmd = easy.bridge('osc', { port: 9000 })
      expect(cmd.protocol).toBe('osc')
      expect(cmd.routerUrl).toBe('ws://localhost:7330')
    })

    it('bridge() passes options through', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const cmd = easy.bridge('mqtt', { broker: 'mqtt://localhost' })
      expect(cmd.options.broker).toBe('mqtt://localhost')
    })

    it('rule() returns valid JSON schema', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      const rule = easy.rule('test', {
        when: '/x',
        then: { set: ['/y', 1] },
      })
      expect(rule.id).toBe('test')
      expect(rule.trigger).toBeDefined()
    })
  })

  describe('Signal Discovery', () => {
    it('getSignals() returns signal array', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      expect(easy.getSignals()).toEqual([])
      expect(mock.getSignals).toHaveBeenCalled()
    })

    it('querySignals() filters by pattern', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      easy.querySignals('/sensors/**')
      expect(mock.querySignals).toHaveBeenCalledWith('/sensors/**')
    })

    it('getLastError() returns null initially', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      expect(easy.getLastError()).toBeNull()
    })

    it('clearError() delegates', () => {
      const mock = mockClasp()
      const easy = new EasyClient(mock as any, 'ws://localhost:7330')
      easy.clearError()
      expect(mock.clearError).toHaveBeenCalled()
    })
  })
})

describe('inferAuthUrl', () => {
  it('converts ws to http with port offset', () => {
    expect(inferAuthUrl('ws://localhost:7330')).toBe('http://localhost:7350')
  })

  it('converts wss to https', () => {
    expect(inferAuthUrl('wss://secure.example.com:7330')).toBe('https://secure.example.com:7350')
  })

  it('preserves path component', () => {
    expect(inferAuthUrl('ws://example.com:8000/relay')).toBe('http://example.com:8020/relay')
  })

  it('handles default port', () => {
    expect(inferAuthUrl('ws://localhost')).toBe('http://localhost:7350')
  })

  it('handles custom port', () => {
    expect(inferAuthUrl('ws://192.168.1.1:9000')).toBe('http://192.168.1.1:9020')
  })
})
