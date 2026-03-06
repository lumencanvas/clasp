import { Clasp, ClaspBuilder } from '@clasp-to/core'
import type { Value, SubscriptionCallback, Unsubscribe, SignalDefinition, ErrorMessage, SubscribeOptions, TimelineKeyframe } from '@clasp-to/core'
import { CryptoClient, MemoryKeyStore } from '@clasp-to/crypto'
import type { ClaspLike } from '@clasp-to/crypto'
import { Device } from './device'
import { Room } from './room'
import { BridgeCommand } from './bridge'
import { buildRuleJSON } from './rules'
import { parseDuration } from './duration'
import type {
  ClaspOptions,
  RegisterOptions,
  GuestOptions,
  LoginOptions,
  RoomOptions,
  RuleDefinition,
  BridgeProtocol,
  BridgeOptions,
} from './types'

/** Default auth port offset from WebSocket port. */
const AUTH_PORT_OFFSET = 20 // 7330 -> 7350

/** Default HTTP request timeout in milliseconds. */
const DEFAULT_HTTP_TIMEOUT = 10_000

/**
 * Infer the auth server URL from the router WebSocket URL.
 * ws://host:7330 -> http://host:7350
 * wss://host:7330/path -> https://host:7350/path
 */
export function inferAuthUrl(wsUrl: string): string {
  const url = new URL(wsUrl)
  const wsPort = parseInt(url.port || '7330', 10)
  const authPort = wsPort + AUTH_PORT_OFFSET
  const protocol = url.protocol === 'wss:' ? 'https:' : 'http:'
  const path = url.pathname !== '/' ? url.pathname : ''
  return `${protocol}//${url.hostname}:${authPort}${path}`
}

/**
 * Create an AbortController with a timeout. Returns the controller
 * and a cleanup function to clear the timer.
 */
function withTimeout(ms: number): { signal: AbortSignal; clear: () => void } {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), ms)
  return { signal: controller.signal, clear: () => clearTimeout(timer) }
}

/**
 * Perform a fetch with timeout and proper error handling for non-JSON responses.
 */
async function safeFetch(url: string, init?: RequestInit & { timeout?: number }): Promise<Response> {
  const timeout = init?.timeout ?? DEFAULT_HTTP_TIMEOUT
  const { signal, clear } = withTimeout(timeout)
  try {
    const res = await fetch(url, { ...init, signal, timeout: undefined } as RequestInit)
    return res
  } finally {
    clear()
  }
}

/**
 * Parse a JSON response body, throwing a descriptive error on failure.
 */
async function safeJSON(res: Response): Promise<Record<string, unknown>> {
  const text = await res.text()
  try {
    return JSON.parse(text)
  } catch {
    throw new Error(`Expected JSON response from ${res.url}, got: ${text.slice(0, 200)}`)
  }
}

/**
 * The main SDK client. Wraps Clasp and optionally CryptoClient
 * to provide a human-friendly API.
 */
export class EasyClient {
  private client: Clasp
  private crypto: CryptoClient | null = null
  private url: string
  private authUrl: string
  private rooms = new Map<string, Room>()
  private roomPending = new Map<string, Promise<Room>>()

  constructor(client: Clasp, url: string, options: ClaspOptions = {}) {
    this.client = client
    this.url = url
    this.authUrl = options.authUrl || inferAuthUrl(url)

    if (options.encrypted) {
      const identityId = options.name || `sdk-${Date.now()}`
      this.crypto = new CryptoClient(client as unknown as ClaspLike, {
        identityId,
        store: new MemoryKeyStore(),
      })
    }
  }

  /** Direct access to the underlying Clasp client. */
  get inner(): Clasp {
    return this.client
  }

  /**
   * Set a persistent parameter value.
   * If encrypted mode is enabled, encrypts the value before sending.
   */
  async set(address: string, value: Value): Promise<void> {
    if (this.crypto) {
      await this.crypto.set(address, value)
      return
    }
    this.client.set(address, value)
  }

  /** Get current value (from cache or server). */
  async get(address: string): Promise<Value> {
    return this.client.get(address)
  }

  /**
   * Subscribe to an address pattern. Auto-decrypts if encrypted mode.
   * Accepts optional subscribe options (maxRate, epsilon, etc.).
   */
  on(pattern: string, callback: SubscriptionCallback, options?: SubscribeOptions): Unsubscribe {
    if (this.crypto) {
      return this.crypto.subscribe(pattern, callback as (data: unknown, address: string) => void)
    }
    return this.client.on(pattern, callback, options)
  }

  /**
   * Subscribe to an address pattern (alias for on()).
   * Provided for symmetry with the core Clasp API.
   */
  subscribe(pattern: string, callback: SubscriptionCallback, options?: SubscribeOptions): Unsubscribe {
    return this.on(pattern, callback, options)
  }

  /**
   * Emit a one-shot event.
   * If encrypted mode is enabled, encrypts the payload before sending.
   */
  async emit(address: string, payload?: Value): Promise<void> {
    if (this.crypto) {
      await this.crypto.emit(address, payload)
      return
    }
    this.client.emit(address, payload)
  }

  /** Send a high-rate stream sample. */
  stream(address: string, value: Value): void {
    this.client.stream(address, value)
  }

  /** Send gesture input. */
  gesture(
    address: string,
    gestureId: number,
    phase: 'start' | 'move' | 'end' | 'cancel',
    payload?: Value
  ): void {
    this.client.gesture(address, gestureId, phase, payload)
  }

  /** Send timeline automation. */
  timeline(
    address: string,
    keyframes: TimelineKeyframe[],
    options?: { loop?: boolean; startTime?: number }
  ): void {
    this.client.timeline(address, keyframes, options)
  }

  /** Send an atomic bundle. */
  bundle(
    messages: Array<{ set?: [string, Value]; emit?: [string, Value] }>,
    options?: { at?: number }
  ): void {
    this.client.bundle(messages, options)
  }

  /** Get server time in microseconds. */
  time(): number {
    return this.client.time()
  }

  /** Check connection status. */
  get connected(): boolean {
    return this.client.connected
  }

  /** Get session ID. */
  get session(): string | null {
    return this.client.session
  }

  /** Get cached value without network request. */
  cached(address: string): Value | undefined {
    return this.client.cached(address)
  }

  /** Get all announced signals. */
  getSignals(): SignalDefinition[] {
    return this.client.getSignals()
  }

  /** Query announced signals matching a pattern. */
  querySignals(pattern: string): SignalDefinition[] {
    return this.client.querySignals(pattern)
  }

  /** Get the last error received from the server. */
  getLastError(): ErrorMessage | null {
    return this.client.getLastError()
  }

  /** Clear the last error. */
  clearError(): void {
    this.client.clearError()
  }

  // --- Device management ---

  /**
   * Register a new device on the auth server.
   * Returns a Device object with the assigned token.
   */
  async register(options: RegisterOptions): Promise<Device> {
    const body: Record<string, unknown> = { scopes: options.scopes }
    if (options.username) body.username = options.username
    if (options.password) body.password = options.password
    if (options.name) body.name = options.name

    const res = await safeFetch(`${this.authUrl}/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    })
    if (!res.ok) {
      const text = await res.text()
      throw new Error(`Registration failed (${res.status}): ${text}`)
    }
    const data = await safeJSON(res)
    return new Device({
      id: (data.session_id || data.user_id) as string,
      token: data.token as string,
      name: options.name,
      scopes: (data.scopes || options.scopes || []) as string[],
      url: this.url,
      authUrl: this.authUrl,
    })
  }

  /** Login with existing credentials. */
  async login(options: LoginOptions): Promise<Device> {
    const res = await safeFetch(`${this.authUrl}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(options),
    })
    if (!res.ok) {
      const text = await res.text()
      throw new Error(`Login failed (${res.status}): ${text}`)
    }
    const data = await safeJSON(res)
    return new Device({
      id: (data.session_id || data.user_id) as string,
      token: data.token as string,
      name: options.username,
      scopes: (data.scopes || []) as string[],
      url: this.url,
      authUrl: this.authUrl,
    })
  }

  /** Create a guest session. */
  async guest(options: GuestOptions = {}): Promise<Device> {
    const res = await safeFetch(`${this.authUrl}/auth/guest`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(options),
    })
    if (!res.ok) {
      const text = await res.text()
      throw new Error(`Guest access failed (${res.status}): ${text}`)
    }
    const data = await safeJSON(res)
    return new Device({
      id: (data.session_id || data.user_id) as string,
      token: data.token as string,
      name: options.name || 'guest',
      scopes: (data.scopes || options.scopes || []) as string[],
      url: this.url,
      authUrl: this.authUrl,
    })
  }

  // --- Encrypted rooms ---

  /**
   * Create or join an encrypted room.
   * All set/emit/on calls through the Room are automatically encrypted.
   * Concurrent calls with the same basePath return the same Room.
   */
  async room(basePath: string, options: RoomOptions = {}): Promise<Room> {
    const existing = this.rooms.get(basePath)
    if (existing) return existing

    // Prevent concurrent room creation for same basePath
    const pending = this.roomPending.get(basePath)
    if (pending) return pending

    const promise = this.createRoom(basePath, options)
    this.roomPending.set(basePath, promise)
    try {
      const room = await promise
      return room
    } finally {
      this.roomPending.delete(basePath)
    }
  }

  /**
   * Destroy a specific room by basePath.
   * Removes it from the client's room map and destroys its session.
   */
  destroyRoom(basePath: string): void {
    const room = this.rooms.get(basePath)
    if (room) {
      room.destroy()
      this.rooms.delete(basePath)
    }
  }

  private async createRoom(basePath: string, options: RoomOptions): Promise<Room> {
    const cryptoClient = this.crypto || new CryptoClient(
      this.client as unknown as ClaspLike,
      {
        identityId: this.client.session || `sdk-${Date.now()}`,
        store: new MemoryKeyStore(),
      }
    )

    let rotationInterval: number | undefined
    if (options.rotateKeys) {
      rotationInterval = parseDuration(options.rotateKeys)
    }

    const session = cryptoClient.session(basePath, {
      passwordHash: options.password,
      rotationInterval,
      onKeyChange: options.onKeyChange,
      onRotation: options.onRotation,
      maxAnnouncementAge: options.maxAnnouncementAge,
    })
    await session.start()
    await session.enableEncryption()

    const r = new Room(basePath, cryptoClient, session)
    this.rooms.set(basePath, r)
    return r
  }

  // --- Bridge helpers ---

  /**
   * Generate bridge CLI command/config for a protocol.
   * Does not spawn a process -- returns a BridgeCommand with .command, .toDockerCompose(), .toEnv().
   */
  bridge(protocol: BridgeProtocol, options: BridgeOptions = {}): BridgeCommand {
    return new BridgeCommand(protocol, this.url, options)
  }

  // --- Rules ---

  /**
   * Define a rule in code. Returns the JSON schema object
   * in the format expected by clasp-rules.
   */
  rule(id: string, definition: RuleDefinition): Record<string, unknown> {
    return buildRuleJSON(id, definition)
  }

  // --- Lifecycle ---

  /** Register connect callback. */
  onConnect(callback: () => void): void {
    this.client.onConnect(callback)
  }

  /** Register disconnect callback. */
  onDisconnect(callback: (reason?: string) => void): void {
    this.client.onDisconnect(callback)
  }

  /** Register error callback. */
  onError(callback: (error: Error) => void): void {
    this.client.onError(callback)
  }

  /** Register reconnect callback. */
  onReconnect(callback: (attempt: number) => void): void {
    this.client.onReconnect(callback)
  }

  /** Close the connection and clean up. Disables reconnect. */
  close(): void {
    for (const room of this.rooms.values()) {
      room.destroy()
    }
    this.rooms.clear()
    this.roomPending.clear()
    this.crypto?.close()
    this.client.close()
  }
}

/**
 * Connect to a CLASP router with minimal configuration.
 *
 * @example
 * ```typescript
 * import clasp from '@clasp-to/sdk'
 *
 * const c = await clasp('ws://localhost:7330')
 * const c = await clasp('ws://localhost:7330', { name: 'My App', token: 'cpsk_...' })
 * const c = await clasp('ws://localhost:7330', { encrypted: true })
 * ```
 */
export default async function clasp(url: string, options: ClaspOptions = {}): Promise<EasyClient> {
  const builder = new ClaspBuilder(url)
  if (options.name) builder.withName(options.name)
  if (options.token) builder.withToken(options.token)
  if (options.reconnect !== undefined) builder.withReconnect(options.reconnect)
  const client = await builder.connect()
  return new EasyClient(client, url, options)
}
