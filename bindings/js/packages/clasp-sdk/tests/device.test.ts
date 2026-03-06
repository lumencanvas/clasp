import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { Device, CredentialBundle } from '../src/device'

function mockFetch(status: number, body: unknown, ok?: boolean) {
  return vi.fn().mockResolvedValue({
    ok: ok ?? (status >= 200 && status < 300),
    status,
    text: vi.fn().mockResolvedValue(typeof body === 'string' ? body : JSON.stringify(body)),
    url: 'http://test',
  })
}

function makeDevice(overrides: Partial<ConstructorParameters<typeof Device>[0]> = {}) {
  return new Device({
    id: 'dev-1',
    token: 'cpsk_parent',
    name: 'Parent Device',
    scopes: ['read:/**', 'write:/**'],
    url: 'ws://localhost:7330',
    authUrl: 'http://localhost:7350',
    ...overrides,
  })
}

describe('Device', () => {
  let originalFetch: typeof globalThis.fetch

  beforeEach(() => {
    originalFetch = globalThis.fetch
  })

  afterEach(() => {
    globalThis.fetch = originalFetch
  })

  describe('Construction', () => {
    it('assigns all fields from DeviceState', () => {
      const device = makeDevice()
      expect(device.id).toBe('dev-1')
      expect(device.token).toBe('cpsk_parent')
      expect(device.name).toBe('Parent Device')
      expect(device.scopes).toEqual(['read:/**', 'write:/**'])
    })

    it('readonly properties', () => {
      const device = makeDevice()
      expect(device.id).toBe('dev-1')
      expect(device.token).toBe('cpsk_parent')
    })
  })

  describe('createChild()', () => {
    it('sends POST to /auth/register with Bearer token', async () => {
      globalThis.fetch = mockFetch(200, {
        session_id: 'child-1',
        token: 'cpsk_child',
        scopes: ['read:/**'],
      }) as any

      const device = makeDevice()
      await device.createChild({ name: 'Child', scopes: ['read:/**'] })

      const call = (globalThis.fetch as any).mock.calls[0]
      expect(call[0]).toBe('http://localhost:7350/auth/register')
      expect(call[1].headers['Authorization']).toBe('Bearer cpsk_parent')
    })

    it('sends correct request body', async () => {
      globalThis.fetch = mockFetch(200, {
        session_id: 'child-1',
        token: 'cpsk_child',
      }) as any

      const device = makeDevice()
      await device.createChild({ name: 'Sensor', scopes: ['write:/sensors/**'] })

      const body = JSON.parse((globalThis.fetch as any).mock.calls[0][1].body)
      expect(body.name).toBe('Sensor')
      expect(body.scopes).toEqual(['write:/sensors/**'])
    })

    it('returns Device with child fields', async () => {
      globalThis.fetch = mockFetch(200, {
        session_id: 'child-1',
        token: 'cpsk_child',
        scopes: ['read:/**'],
      }) as any

      const device = makeDevice()
      const child = await device.createChild({ name: 'Child', scopes: ['read:/**'] })

      expect(child.id).toBe('child-1')
      expect(child.token).toBe('cpsk_child')
      expect(child.name).toBe('Child')
    })

    it('throws on 403 Forbidden', async () => {
      globalThis.fetch = mockFetch(403, 'Scope violation', false) as any
      const device = makeDevice()
      await expect(device.createChild({ name: 'X', scopes: ['admin:/**'] }))
        .rejects.toThrow('Failed to create child device (403)')
    })

    it('throws on 409 Conflict', async () => {
      globalThis.fetch = mockFetch(409, 'Already exists', false) as any
      const device = makeDevice()
      await expect(device.createChild({ name: 'Dup', scopes: [] }))
        .rejects.toThrow('Failed to create child device (409)')
    })

    it('throws on non-JSON response body', async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        text: vi.fn().mockResolvedValue('<html>Bad Gateway</html>'),
        url: 'http://test',
      }) as any

      const device = makeDevice()
      await expect(device.createChild({ name: 'X', scopes: [] }))
        .rejects.toThrow('Expected JSON')
    })
  })

  describe('provision()', () => {
    it('creates child and returns CredentialBundle', async () => {
      globalThis.fetch = mockFetch(200, {
        session_id: 'prov-1',
        token: 'cpsk_provisioned',
        scopes: ['write:/sensors/**'],
      }) as any

      const device = makeDevice()
      const creds = await device.provision({
        name: 'Sensor',
        scopes: ['write:/sensors/**'],
      })

      expect(creds).toBeInstanceOf(CredentialBundle)
      expect(creds.token).toBe('cpsk_provisioned')
      expect(creds.name).toBe('Sensor')
    })

    it('CredentialBundle has correct fields', async () => {
      globalThis.fetch = mockFetch(200, {
        session_id: 'p1',
        token: 'cpsk_x',
      }) as any

      const device = makeDevice()
      const creds = await device.provision({
        name: 'Test',
        scopes: ['read:/**'],
      })

      expect(creds.url).toBe('ws://localhost:7330')
      expect(creds.scopes).toEqual(['read:/**'])
    })

    it('with expires: "30d" calculates correct ISO date', async () => {
      globalThis.fetch = mockFetch(200, {
        session_id: 'p1',
        token: 'cpsk_x',
      }) as any

      const now = Date.now()
      vi.spyOn(Date, 'now').mockReturnValue(now)

      const device = makeDevice()
      const creds = await device.provision({
        name: 'Expiring',
        scopes: [],
        expires: '30d',
      })

      const expected = new Date(now + 30 * 86_400_000).toISOString()
      expect(creds.expires).toBe(expected)

      vi.restoreAllMocks()
    })

    it('with expires: "1h" calculates correct ISO date', async () => {
      globalThis.fetch = mockFetch(200, {
        session_id: 'p1',
        token: 'cpsk_x',
      }) as any

      const now = Date.now()
      vi.spyOn(Date, 'now').mockReturnValue(now)

      const device = makeDevice()
      const creds = await device.provision({
        name: 'ShortLived',
        scopes: [],
        expires: '1h',
      })

      const expected = new Date(now + 3_600_000).toISOString()
      expect(creds.expires).toBe(expected)

      vi.restoreAllMocks()
    })

    it('without expires, CredentialBundle.expires is undefined', async () => {
      globalThis.fetch = mockFetch(200, {
        session_id: 'p1',
        token: 'cpsk_x',
      }) as any

      const device = makeDevice()
      const creds = await device.provision({ name: 'NoExp', scopes: [] })
      expect(creds.expires).toBeUndefined()
    })
  })

  describe('provisionBatch()', () => {
    it('provisions multiple devices in parallel', async () => {
      let callCount = 0
      globalThis.fetch = vi.fn().mockImplementation(async () => {
        callCount++
        return {
          ok: true,
          status: 200,
          text: vi.fn().mockResolvedValue(JSON.stringify({
            session_id: `batch-${callCount}`,
            token: `cpsk_batch_${callCount}`,
          })),
          url: 'http://test',
        }
      }) as any

      const device = makeDevice()
      const results = await device.provisionBatch([
        { name: 'A', scopes: ['read:/**'] },
        { name: 'B', scopes: ['write:/**'] },
      ])

      expect(results).toHaveLength(2)
      expect(results[0]).toBeInstanceOf(CredentialBundle)
      expect(results[1]).toBeInstanceOf(CredentialBundle)
    })

    it('empty array returns empty array', async () => {
      const device = makeDevice()
      const results = await device.provisionBatch([])
      expect(results).toEqual([])
    })
  })

  describe('revoke()', () => {
    it('sends PUT to /api/entities/{id}/status', async () => {
      globalThis.fetch = mockFetch(200, {}) as any
      const device = makeDevice()
      await device.revoke('child-123')

      const call = (globalThis.fetch as any).mock.calls[0]
      expect(call[0]).toBe('http://localhost:7350/api/entities/child-123/status')
      expect(call[1].method).toBe('PUT')
      expect(call[1].headers['Authorization']).toBe('Bearer cpsk_parent')

      const body = JSON.parse(call[1].body)
      expect(body.status).toBe('revoked')
    })

    it('throws on 404 Not Found', async () => {
      globalThis.fetch = mockFetch(404, 'Not found', false) as any
      const device = makeDevice()
      await expect(device.revoke('nonexistent')).rejects.toThrow('Failed to revoke device (404)')
    })

    it('throws on 403 Forbidden', async () => {
      globalThis.fetch = mockFetch(403, 'Forbidden', false) as any
      const device = makeDevice()
      await expect(device.revoke('child-1')).rejects.toThrow('Failed to revoke device (403)')
    })
  })

  describe('children()', () => {
    it('sends GET to /api/entities with Bearer token', async () => {
      globalThis.fetch = mockFetch(200, []) as any
      const device = makeDevice()
      await device.children()

      const call = (globalThis.fetch as any).mock.calls[0]
      expect(call[0]).toBe('http://localhost:7350/api/entities')
      expect(call[1].headers['Authorization']).toBe('Bearer cpsk_parent')
    })

    it('parses array response format', async () => {
      globalThis.fetch = mockFetch(200, [
        { id: 'c1', name: 'Child 1', scopes: ['read:/**'] },
        { id: 'c2', name: 'Child 2', scopes: [] },
      ]) as any

      const device = makeDevice()
      const children = await device.children()
      expect(children).toHaveLength(2)
      expect(children[0].id).toBe('c1')
      expect(children[1].name).toBe('Child 2')
    })

    it('parses { entities: [...] } response format', async () => {
      globalThis.fetch = mockFetch(200, {
        entities: [{ id: 'c1', name: 'Child', scopes: [] }],
      }) as any

      const device = makeDevice()
      const children = await device.children()
      expect(children).toHaveLength(1)
    })

    it('returns Device objects with empty tokens', async () => {
      globalThis.fetch = mockFetch(200, [
        { id: 'c1', name: 'Child', scopes: ['read:/**'] },
      ]) as any

      const device = makeDevice()
      const children = await device.children()
      expect(children[0].token).toBe('')
    })

    it('throws on 403 Forbidden', async () => {
      globalThis.fetch = mockFetch(403, 'Forbidden', false) as any
      const device = makeDevice()
      await expect(device.children()).rejects.toThrow('Failed to list devices (403)')
    })

    it('returns empty array for empty response', async () => {
      globalThis.fetch = mockFetch(200, []) as any
      const device = makeDevice()
      const children = await device.children()
      expect(children).toEqual([])
    })
  })
})

describe('CredentialBundle', () => {
  it('toJSON produces valid JSON', () => {
    const creds = new CredentialBundle({
      token: 'cpsk_test123',
      url: 'ws://192.168.1.50:7330',
      name: 'Kitchen Sensor',
      scopes: ['write:/sensors/kitchen/**'],
      expires: '2026-04-05T00:00:00.000Z',
    })

    const json = JSON.parse(creds.toJSON())
    expect(json.token).toBe('cpsk_test123')
    expect(json.url).toBe('ws://192.168.1.50:7330')
    expect(json.name).toBe('Kitchen Sensor')
    expect(json.scopes).toEqual(['write:/sensors/kitchen/**'])
    expect(json.expires).toBe('2026-04-05T00:00:00.000Z')
  })

  it('toEnv produces environment variable format', () => {
    const creds = new CredentialBundle({
      token: 'cpsk_test123',
      url: 'ws://192.168.1.50:7330',
      name: 'Kitchen Sensor',
      scopes: ['write:/sensors/kitchen/**'],
    })

    const env = creds.toEnv()
    expect(env).toContain('CLASP_URL=ws://192.168.1.50:7330')
    expect(env).toContain('CLASP_TOKEN=cpsk_test123')
    expect(env).toContain('CLASP_NAME=Kitchen Sensor')
    expect(env).toContain('CLASP_SCOPES=write:/sensors/kitchen/**')
  })

  it('toJSON omits expires when not set', () => {
    const creds = new CredentialBundle({
      token: 'cpsk_x',
      url: 'ws://localhost:7330',
      name: 'Test',
      scopes: [],
    })

    const json = JSON.parse(creds.toJSON())
    expect(json.expires).toBeUndefined()
  })

  it('toEnv omits scopes when empty', () => {
    const creds = new CredentialBundle({
      token: 'cpsk_x',
      url: 'ws://localhost:7330',
      name: 'Test',
      scopes: [],
    })

    expect(creds.toEnv()).not.toContain('CLASP_SCOPES')
  })

  it('toEnv includes expires when set', () => {
    const creds = new CredentialBundle({
      token: 'cpsk_x',
      url: 'ws://localhost:7330',
      name: 'Test',
      scopes: [],
      expires: '2026-12-31T00:00:00Z',
    })

    expect(creds.toEnv()).toContain('CLASP_EXPIRES=2026-12-31T00:00:00Z')
  })
})
