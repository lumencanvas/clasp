import { describe, it, expect, vi, beforeEach } from 'vitest'
import { EventEmitter } from 'events'
import type { ChildProcess } from 'child_process'

// Mock child_process.spawn
const mockSpawn = vi.fn()
vi.mock('child_process', () => ({
  spawn: (...args: any[]) => mockSpawn(...args),
  execFileSync: vi.fn(),
}))

// Mock binary resolution to always succeed
vi.mock('../src/binary', () => ({
  resolveBinary: vi.fn().mockReturnValue('/usr/bin/clasp-relay'),
}))

// Mock health check to resolve immediately
vi.mock('../src/health', () => ({
  waitForReady: vi.fn().mockResolvedValue(undefined),
  parsePortFromOutput: vi.fn(),
}))

// Mock fs operations
vi.mock('fs/promises', () => ({
  writeFile: vi.fn().mockResolvedValue(undefined),
  unlink: vi.fn().mockResolvedValue(undefined),
  mkdtemp: vi.fn().mockResolvedValue('/tmp/clasp-relay-test'),
  rm: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('fs', () => ({
  accessSync: vi.fn(),
  constants: { X_OK: 1 },
}))

const { createRelay, RelayServer } = await import('../src/server')
const { writeFile, mkdtemp } = await import('fs/promises')

function createMockProcess(): ChildProcess {
  const proc = new EventEmitter() as any
  proc.pid = 12345
  proc.stdout = new EventEmitter()
  proc.stderr = new EventEmitter()
  proc.kill = vi.fn()
  proc.stdin = null
  return proc
}

describe('createRelay', () => {
  let mockProc: ChildProcess

  beforeEach(() => {
    vi.clearAllMocks()
    mockProc = createMockProcess()
    mockSpawn.mockReturnValue(mockProc)
  })

  it('creates a relay with default config', async () => {
    const server = await createRelay()
    expect(server).toBeInstanceOf(RelayServer)
    expect(server.pid).toBe(12345)
    expect(server.url).toBe('ws://localhost:7330')
    expect(server.authUrl).toBeNull()
    expect(server.healthUrl).toBeNull()
    expect(server.stopped).toBe(false)
  })

  it('creates a relay with config object', async () => {
    const server = await createRelay({ port: 8080, authPort: 8081, healthPort: 8082 })
    expect(server.url).toBe('ws://localhost:8080')
    expect(server.authUrl).toBe('http://localhost:8081')
    expect(server.healthUrl).toBe('http://localhost:8082')
    expect(mockSpawn).toHaveBeenCalledWith(
      '/usr/bin/clasp-relay',
      expect.arrayContaining(['--ws-port', '8080', '--auth-port', '8081', '--health-port', '8082']),
      expect.any(Object),
    )
  })

  it('creates a relay with builder callback', async () => {
    const server = await createRelay(r => r.port(9000).verbose())
    expect(server.url).toBe('ws://localhost:9000')
    expect(mockSpawn).toHaveBeenCalledWith(
      '/usr/bin/clasp-relay',
      expect.arrayContaining(['--ws-port', '9000', '--verbose']),
      expect.any(Object),
    )
  })

  it('passes spawn options (cwd, env)', async () => {
    await createRelay({ port: 7330 }, { cwd: '/tmp', env: { FOO: 'bar' } })
    const spawnCall = mockSpawn.mock.calls[0]
    expect(spawnCall[2].cwd).toBe('/tmp')
    expect(spawnCall[2].env.FOO).toBe('bar')
  })

  it('uses inherit stdio when option set', async () => {
    await createRelay({ port: 7330 }, { inherit: true })
    const spawnCall = mockSpawn.mock.calls[0]
    expect(spawnCall[2].stdio).toBe('inherit')
  })

  it('uses pipe stdio by default', async () => {
    await createRelay({ port: 7330 })
    const spawnCall = mockSpawn.mock.calls[0]
    expect(spawnCall[2].stdio).toEqual(['ignore', 'pipe', 'pipe'])
  })

  it('resolves host 0.0.0.0 to localhost in URL', async () => {
    const server = await createRelay({ port: 7330, host: '0.0.0.0' })
    expect(server.url).toBe('ws://localhost:7330')
  })

  it('resolves host 127.0.0.1 to localhost in URL', async () => {
    const server = await createRelay({ port: 7330, host: '127.0.0.1' })
    expect(server.url).toBe('ws://localhost:7330')
  })

  it('uses custom host in URL', async () => {
    const server = await createRelay({ port: 7330, host: '192.168.1.100' })
    expect(server.url).toBe('ws://192.168.1.100:7330')
  })

  it('writes inline appConfig to temp file', async () => {
    const appConfig = { scopes: ['read:/**'] }
    await createRelay({ port: 7330, appConfig })
    expect(mkdtemp).toHaveBeenCalled()
    expect(writeFile).toHaveBeenCalledWith(
      '/tmp/clasp-relay-test/app-config.json',
      JSON.stringify(appConfig, null, 2),
    )
    expect(mockSpawn.mock.calls[0][1]).toEqual(
      expect.arrayContaining(['--app-config', '/tmp/clasp-relay-test/app-config.json']),
    )
  })

  it('writes inline rules to temp file', async () => {
    const rules = { triggers: [{ event: 'param:set' }] }
    await createRelay({ port: 7330, rules })
    expect(writeFile).toHaveBeenCalledWith(
      '/tmp/clasp-relay-test/rules.json',
      JSON.stringify(rules, null, 2),
    )
  })

  it('passes string appConfig path directly without temp file', async () => {
    await createRelay({ port: 7330, appConfig: '/etc/clasp/app.json' })
    expect(writeFile).not.toHaveBeenCalled()
    expect(mockSpawn.mock.calls[0][1]).toEqual(
      expect.arrayContaining(['--app-config', '/etc/clasp/app.json']),
    )
  })

  it('throws if callback does not return RelayBuilder', async () => {
    await expect(
      createRelay((() => 'not a builder') as any)
    ).rejects.toThrow('must return a RelayBuilder')
  })

  it('skips readiness check when inherit is true', async () => {
    const { waitForReady } = await import('../src/health')
    await createRelay({ port: 7330 }, { inherit: true })
    expect(waitForReady).not.toHaveBeenCalled()
  })
})

describe('RelayServer lifecycle', () => {
  let mockProc: ChildProcess

  beforeEach(() => {
    vi.clearAllMocks()
    mockProc = createMockProcess()
    mockSpawn.mockReturnValue(mockProc)
  })

  it('captures stdout as log events', async () => {
    const server = await createRelay({ port: 7330 })
    const logs: string[] = []
    server.on('log', (line: string) => logs.push(line))

    mockProc.stdout!.emit('data', Buffer.from('line 1\nline 2\n'))
    expect(logs).toEqual(['line 1', 'line 2'])
  })

  it('captures stderr as log events', async () => {
    const server = await createRelay({ port: 7330 })
    const logs: string[] = []
    server.on('log', (line: string) => logs.push(line))

    mockProc.stderr!.emit('data', Buffer.from('error line\n'))
    expect(logs).toEqual(['error line'])
  })

  it('buffers logs up to 500 lines', async () => {
    const server = await createRelay({ port: 7330 })
    for (let i = 0; i < 600; i++) {
      mockProc.stdout!.emit('data', Buffer.from(`line ${i}\n`))
    }
    expect(server.logs.length).toBe(500)
    expect(server.logs[0]).toBe('line 100')
    expect(server.logs[499]).toBe('line 599')
  })

  it('emits exit event with code and signal', async () => {
    const server = await createRelay({ port: 7330 })
    const exits: Array<[number | null, string | null]> = []
    server.on('exit', (code: number | null, signal: string | null) => exits.push([code, signal]))

    mockProc.emit('exit', 0, null)
    expect(exits).toEqual([[0, null]])
    expect(server.stopped).toBe(true)
    expect(server.exitCode).toBe(0)
    expect(server.exitSignal).toBeNull()
  })

  it('stores exit signal', async () => {
    const server = await createRelay({ port: 7330 })
    mockProc.emit('exit', null, 'SIGTERM')
    expect(server.exitCode).toBeNull()
    expect(server.exitSignal).toBe('SIGTERM')
  })

  it('emits error event', async () => {
    const server = await createRelay({ port: 7330 })
    const errors: Error[] = []
    server.on('error', (err: Error) => errors.push(err))

    mockProc.emit('error', new Error('spawn failed'))
    expect(errors[0].message).toBe('spawn failed')
  })

  it('exposes underlying child process', async () => {
    const server = await createRelay({ port: 7330 })
    expect(server.process).toBe(mockProc)
  })

  it('stop sends SIGTERM then resolves on exit', async () => {
    const server = await createRelay({ port: 7330 })
    const stopPromise = server.stop()

    expect(mockProc.kill).toHaveBeenCalledWith('SIGTERM')
    mockProc.emit('exit', 0, null)
    await stopPromise
    expect(server.stopped).toBe(true)
  })

  it('stop force-kills after timeout', async () => {
    vi.useFakeTimers()
    const server = await createRelay({ port: 7330 })
    const stopPromise = server.stop(100)

    expect(mockProc.kill).toHaveBeenCalledWith('SIGTERM')
    vi.advanceTimersByTime(150)
    expect(mockProc.kill).toHaveBeenCalledWith('SIGKILL')

    mockProc.emit('exit', null, 'SIGKILL')
    await stopPromise
    vi.useRealTimers()
  })

  it('stop is idempotent on already-stopped server', async () => {
    const server = await createRelay({ port: 7330 })
    mockProc.emit('exit', 0, null)
    await server.stop()
    expect(mockProc.kill).not.toHaveBeenCalled()
  })

  it('kill sends SIGKILL', async () => {
    const server = await createRelay({ port: 7330 })
    server.kill()
    expect(mockProc.kill).toHaveBeenCalledWith('SIGKILL')
  })

  it('kill is idempotent on already-stopped server', async () => {
    const server = await createRelay({ port: 7330 })
    mockProc.emit('exit', 0, null)
    server.kill()
    expect(mockProc.kill).not.toHaveBeenCalled()
  })
})

describe('config to args conversion', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockSpawn.mockReturnValue(createMockProcess())
  })

  it('converts full config to correct args', async () => {
    await createRelay({
      port: 7330,
      host: '0.0.0.0',
      name: 'Test Relay',
      authPort: 7350,
      verbose: true,
      maxSessions: 100,
      sessionTimeout: 3600,
      paramTtl: 60,
      signalTtl: 30,
      drainTimeout: 10,
      healthPort: 7360,
    })

    const args = mockSpawn.mock.calls[0][1] as string[]
    expect(args).toContain('--ws-port')
    expect(args).toContain('7330')
    expect(args).toContain('--host')
    expect(args).toContain('0.0.0.0')
    expect(args).toContain('--name')
    expect(args).toContain('Test Relay')
    expect(args).toContain('--auth-port')
    expect(args).toContain('7350')
    expect(args).toContain('--verbose')
    expect(args).toContain('--max-sessions')
    expect(args).toContain('100')
    expect(args).toContain('--drain-timeout')
    expect(args).toContain('10')
    expect(args).toContain('--health-port')
    expect(args).toContain('7360')
  })

  it('handles persist config', async () => {
    await createRelay({
      port: 7330,
      persist: { path: './state.db', interval: 5 },
    })
    const args = mockSpawn.mock.calls[0][1] as string[]
    expect(args).toContain('--persist')
    expect(args).toContain('./state.db')
    expect(args).toContain('--persist-interval')
    expect(args).toContain('5')
  })

  it('handles journal memory mode', async () => {
    await createRelay({ port: 7330, journal: 'memory' })
    const args = mockSpawn.mock.calls[0][1] as string[]
    expect(args).toContain('--journal-memory')
  })

  it('handles CORS origins', async () => {
    await createRelay({ port: 7330, corsOrigin: ['http://a.com', 'http://b.com'] })
    const args = mockSpawn.mock.calls[0][1] as string[]
    expect(args).toContain('--cors-origin')
    expect(args).toContain('http://a.com,http://b.com')
  })

  it('handles noTtl flag', async () => {
    await createRelay({ port: 7330, noTtl: true })
    const args = mockSpawn.mock.calls[0][1] as string[]
    expect(args).toContain('--no-ttl')
  })

  it('logLevel takes priority over verbose', async () => {
    await createRelay({ port: 7330, logLevel: 'debug', verbose: true })
    const args = mockSpawn.mock.calls[0][1] as string[]
    expect(args).toContain('--log-level')
    expect(args).toContain('debug')
    expect(args).not.toContain('--verbose')
  })
})
