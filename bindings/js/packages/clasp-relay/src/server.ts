import { RelayBuilder } from '@clasp-to/sdk'
import { resolveBinary } from './binary'
import { waitForReady, parsePortFromOutput } from './health'
import type { RelayServerOptions, ConfigInput } from './types'
import { spawn, type ChildProcess } from 'child_process'
import { EventEmitter } from 'events'
import { writeFile, unlink, mkdtemp, rm } from 'fs/promises'
import { tmpdir } from 'os'
import { join } from 'path'

const MAX_LOG_BUFFER = 500

export class RelayServer extends EventEmitter {
  readonly pid: number
  private _url: string
  private _authUrl: string | null
  private _healthUrl: string | null
  private _process: ChildProcess
  private _tempDir: string | null = null
  private _tempFiles: string[] = []
  private _logs: string[] = []
  private _stopped = false
  private _exitCode: number | null = null
  private _exitSignal: string | null = null

  /** @internal */
  constructor(
    proc: ChildProcess,
    url: string,
    authUrl: string | null,
    healthUrl: string | null,
    tempDir: string | null,
    tempFiles: string[],
  ) {
    super()
    this._process = proc
    this.pid = proc.pid!
    this._url = url
    this._authUrl = authUrl
    this._healthUrl = healthUrl
    this._tempDir = tempDir
    this._tempFiles = tempFiles

    proc.stdout?.on('data', (data: Buffer) => {
      const lines = data.toString().split('\n').filter(Boolean)
      for (const line of lines) {
        this._logs.push(line)
        if (this._logs.length > MAX_LOG_BUFFER) this._logs.shift()
        this.emit('log', line)
      }
    })

    proc.stderr?.on('data', (data: Buffer) => {
      const lines = data.toString().split('\n').filter(Boolean)
      for (const line of lines) {
        this._logs.push(line)
        if (this._logs.length > MAX_LOG_BUFFER) this._logs.shift()
        this.emit('log', line)
      }
    })

    proc.on('error', (err) => {
      this.emit('error', err)
    })

    proc.on('exit', (code, signal) => {
      this._exitCode = code
      this._exitSignal = signal
      this._stopped = true
      this.emit('exit', code, signal)
      this._cleanup()
    })
  }

  /** WebSocket URL of the running relay. */
  get url(): string {
    return this._url
  }

  /** Auth HTTP URL, or null if no auth port was configured. */
  get authUrl(): string | null {
    return this._authUrl
  }

  /** Health check HTTP URL, or null if no health port was configured. */
  get healthUrl(): string | null {
    return this._healthUrl
  }

  /** Whether the relay process has exited. */
  get stopped(): boolean {
    return this._stopped
  }

  /** Exit code of the process, or null if still running. */
  get exitCode(): number | null {
    return this._exitCode
  }

  /** Exit signal that terminated the process, or null. */
  get exitSignal(): string | null {
    return this._exitSignal
  }

  /** Recent log lines (up to 500). */
  get logs(): readonly string[] {
    return this._logs
  }

  /** The underlying child process. */
  get process(): ChildProcess {
    return this._process
  }

  /** Gracefully stop the relay (SIGTERM), then SIGKILL after timeout. */
  async stop(timeout = 5000): Promise<void> {
    if (this._stopped) return

    return new Promise<void>((resolve) => {
      const onExit = () => {
        clearTimeout(timer)
        resolve()
      }

      const timer = setTimeout(() => {
        // Force kill and wait for actual exit
        this._process.kill('SIGKILL')
      }, timeout)

      this._process.once('exit', onExit)
      this._process.kill('SIGTERM')
    })
  }

  /** Force-kill the relay process (SIGKILL). */
  kill(): void {
    if (this._stopped) return
    this._process.kill('SIGKILL')
  }

  /** @internal */
  _setUrl(url: string): void {
    this._url = url
  }

  private async _cleanup(): Promise<void> {
    for (const f of this._tempFiles) {
      try { await unlink(f) } catch { /* ignore */ }
    }
    if (this._tempDir) {
      try {
        await rm(this._tempDir, { recursive: true, force: true })
      } catch { /* ignore */ }
    }
  }
}

/**
 * Create and start a CLASP relay server.
 *
 * @example
 * ```typescript
 * // Simple -- starts on a specific port
 * const relay = await createRelay({ port: 7330 })
 *
 * // Builder callback -- full fluent API
 * const relay = await createRelay(r => r
 *   .port(7330)
 *   .authPort(7350)
 *   .name('My Relay')
 *   .verbose()
 * )
 *
 * console.log(relay.url)  // ws://localhost:7330
 * await relay.stop()
 * ```
 */
export async function createRelay(
  config: ConfigInput = {},
  options: RelayServerOptions = {},
): Promise<RelayServer> {
  const builder = typeof config === 'function'
    ? config(new RelayBuilder())
    : RelayBuilder.fromConfig(config)

  if (!(builder instanceof RelayBuilder)) {
    throw new TypeError(
      'Config callback must return a RelayBuilder (return the builder from the callback)'
    )
  }

  const binary = resolveBinary(options.binary)
  const relayConfig = builder.getConfig()

  // Write inline appConfig/rules to temp files
  let tempDir: string | null = null
  const tempFiles: string[] = []
  const extraArgs: string[] = []

  if (relayConfig.appConfig && typeof relayConfig.appConfig !== 'string') {
    tempDir = tempDir || await mkdtemp(join(tmpdir(), 'clasp-relay-'))
    const configPath = join(tempDir, 'app-config.json')
    await writeFile(configPath, JSON.stringify(relayConfig.appConfig, null, 2))
    tempFiles.push(configPath)
    extraArgs.push('--app-config', configPath)
  }

  if (relayConfig.rules && typeof relayConfig.rules !== 'string') {
    tempDir = tempDir || await mkdtemp(join(tmpdir(), 'clasp-relay-'))
    const rulesPath = join(tempDir, 'rules.json')
    await writeFile(rulesPath, JSON.stringify(relayConfig.rules, null, 2))
    tempFiles.push(rulesPath)
    extraArgs.push('--rules', rulesPath)
  }

  const args = [...builder.toArgs(), ...extraArgs]
  const wsPort = relayConfig.port || 7330
  const host = relayConfig.host
  const urlHost = !host || host === '0.0.0.0' || host === '127.0.0.1' ? 'localhost' : host
  const authPort = relayConfig.authPort
  const healthPort = relayConfig.healthPort
  const usePort0 = wsPort === 0

  const proc = spawn(binary, args, {
    cwd: options.cwd,
    env: { ...process.env, ...options.env },
    stdio: options.inherit ? 'inherit' : ['ignore', 'pipe', 'pipe'],
  })

  if (!proc.pid) {
    throw new Error(`Failed to spawn clasp-relay: process has no PID`)
  }

  const wsUrl = usePort0 ? 'ws://localhost:0' : `ws://${urlHost}:${wsPort}`
  const authUrl = authPort ? `http://${urlHost}:${authPort}` : null
  const healthUrl = healthPort ? `http://${urlHost}:${healthPort}` : null

  const server = new RelayServer(proc, wsUrl, authUrl, healthUrl, tempDir, tempFiles)

  if (options.inherit) {
    // Can't do readiness detection with inherited stdio
    return server
  }

  // For port 0, detect actual port from output
  if (usePort0) {
    const portDetected = new Promise<number>((resolve, reject) => {
      const onLog = (line: string) => {
        const port = parsePortFromOutput(line)
        if (port) {
          server.removeListener('log', onLog)
          resolve(port)
        }
      }
      const onExit = (code: number | null) => {
        server.removeListener('log', onLog)
        reject(new Error(`Relay exited with code ${code} before becoming ready`))
      }
      server.on('log', onLog)
      server.once('exit', onExit)
    })

    try {
      const actualPort = await Promise.race([
        portDetected,
        new Promise<never>((_, reject) =>
          setTimeout(() => reject(new Error('Timed out detecting port')), options.readyTimeout ?? 10_000)
        ),
      ])
      server._setUrl(`ws://localhost:${actualPort}`)
    } catch (err) {
      server.kill()
      throw err
    }
  }

  // Wait for readiness using the best available probe
  const effectiveWsPort = usePort0
    ? parseInt(server.url.split(':').pop()!, 10)
    : wsPort

  if (effectiveWsPort) {
    const readyPromise = waitForReady(
      effectiveWsPort,
      healthPort,
      authPort,
      'localhost',
      options.readyTimeout ?? 10_000,
    )

    const earlyExit = new Promise<never>((_, reject) => {
      const onExit = (code: number | null) => {
        reject(new Error(
          `Relay exited with code ${code} before becoming ready.\n` +
          `Last logs:\n${server.logs.slice(-10).join('\n')}`
        ))
      }
      server.once('exit', onExit)
      // Clean up listener if readiness succeeds
      readyPromise.then(() => server.removeListener('exit', onExit), () => {})
    })

    try {
      await Promise.race([readyPromise, earlyExit])
    } catch (err) {
      server.kill()
      throw err
    }

    server.emit('ready')
  }

  return server
}
