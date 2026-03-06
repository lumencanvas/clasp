import { ClaspBuilder } from '@clasp-to/core'
import { EasyClient } from './easy'
import { parseDuration } from './duration'
import type { ChildDeviceOptions, ProvisionOptions } from './types'

/** Default HTTP request timeout in milliseconds. */
const DEFAULT_HTTP_TIMEOUT = 10_000

/** Internal state for a Device. */
interface DeviceState {
  id: string
  token: string
  name?: string
  scopes: string[]
  url: string
  authUrl: string
}

/**
 * Perform a fetch with timeout.
 */
async function fetchWithTimeout(url: string, init?: RequestInit, timeout = DEFAULT_HTTP_TIMEOUT): Promise<Response> {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), timeout)
  try {
    return await fetch(url, { ...init, signal: controller.signal })
  } finally {
    clearTimeout(timer)
  }
}

/**
 * Parse JSON from a response, with a guard for non-JSON bodies.
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
 * Credential bundle for a provisioned device.
 * Contains everything needed to connect as that device.
 */
export class CredentialBundle {
  readonly token: string
  readonly url: string
  readonly name: string
  readonly scopes: string[]
  readonly expires?: string

  constructor(data: { token: string; url: string; name: string; scopes: string[]; expires?: string }) {
    this.token = data.token
    this.url = data.url
    this.name = data.name
    this.scopes = data.scopes
    this.expires = data.expires
  }

  /** Serialize to JSON (for QR codes, config files). */
  toJSON(): string {
    return JSON.stringify({
      token: this.token,
      url: this.url,
      name: this.name,
      scopes: this.scopes,
      ...(this.expires ? { expires: this.expires } : {}),
    })
  }

  /** Serialize to environment variable format. */
  toEnv(): string {
    const lines = [
      `CLASP_URL=${this.url}`,
      `CLASP_TOKEN=${this.token}`,
      `CLASP_NAME=${this.name}`,
    ]
    if (this.scopes.length > 0) {
      lines.push(`CLASP_SCOPES=${this.scopes.join(',')}`)
    }
    if (this.expires) {
      lines.push(`CLASP_EXPIRES=${this.expires}`)
    }
    return lines.join('\n')
  }

  /** Connect to the router as this device. */
  async connect(): Promise<EasyClient> {
    const builder = new ClaspBuilder(this.url)
    builder.withName(this.name)
    builder.withToken(this.token)
    const client = await builder.connect()
    return new EasyClient(client, this.url)
  }
}

/**
 * Represents a registered device with identity, token, and scopes.
 * Can create child devices and provision credentials.
 */
export class Device {
  readonly id: string
  readonly token: string
  readonly name: string | undefined
  readonly scopes: string[]

  private url: string
  private authUrl: string

  constructor(state: DeviceState) {
    this.id = state.id
    this.token = state.token
    this.name = state.name
    this.scopes = state.scopes
    this.url = state.url
    this.authUrl = state.authUrl
  }

  /**
   * Create a child device with narrower scopes.
   * Registers a new device on the auth server with this device's token as authorization.
   */
  async createChild(options: ChildDeviceOptions): Promise<Device> {
    const res = await fetchWithTimeout(`${this.authUrl}/auth/register`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.token}`,
      },
      body: JSON.stringify({
        name: options.name,
        scopes: options.scopes,
      }),
    })
    if (!res.ok) {
      const text = await res.text()
      throw new Error(`Failed to create child device (${res.status}): ${text}`)
    }
    const data = await safeJSON(res)
    return new Device({
      id: (data.session_id || data.user_id) as string,
      token: data.token as string,
      name: options.name,
      scopes: (data.scopes || options.scopes) as string[],
      url: this.url,
      authUrl: this.authUrl,
    })
  }

  /**
   * Connect to the router as this device.
   */
  async connect(): Promise<EasyClient> {
    const builder = new ClaspBuilder(this.url)
    if (this.name) builder.withName(this.name)
    builder.withToken(this.token)
    const client = await builder.connect()
    return new EasyClient(client, this.url)
  }

  /**
   * Provision a credential bundle for a child device.
   * Creates the device and returns a Credentials object with toJSON/toEnv/connect.
   */
  async provision(options: ProvisionOptions): Promise<CredentialBundle> {
    const child = await this.createChild({
      name: options.name,
      scopes: options.scopes,
    })

    let expires: string | undefined
    if (options.expires) {
      // Calculate absolute expiration from relative duration
      const ms = parseDuration(options.expires)
      expires = new Date(Date.now() + ms).toISOString()
    }

    return new CredentialBundle({
      token: child.token,
      url: this.url,
      name: options.name,
      scopes: options.scopes,
      expires,
    })
  }

  /**
   * Provision multiple devices in parallel.
   */
  async provisionBatch(
    devices: Array<{ name: string; scopes: string[] }>
  ): Promise<CredentialBundle[]> {
    return Promise.all(
      devices.map(d => this.provision({ name: d.name, scopes: d.scopes }))
    )
  }

  /**
   * Revoke a child device's access.
   */
  async revoke(childId: string): Promise<void> {
    const res = await fetchWithTimeout(`${this.authUrl}/api/entities/${childId}/status`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.token}`,
      },
      body: JSON.stringify({ status: 'revoked' }),
    })
    if (!res.ok) {
      const text = await res.text()
      throw new Error(`Failed to revoke device (${res.status}): ${text}`)
    }
  }

  /**
   * List child devices (if entity registry is available).
   */
  async children(): Promise<Device[]> {
    const res = await fetchWithTimeout(`${this.authUrl}/api/entities`, {
      headers: {
        'Authorization': `Bearer ${this.token}`,
      },
    })
    if (!res.ok) {
      const text = await res.text()
      throw new Error(`Failed to list devices (${res.status}): ${text}`)
    }
    const data = await safeJSON(res)
    const entities = Array.isArray(data) ? data : (data.entities || []) as Record<string, unknown>[]
    return entities.map((e: Record<string, unknown>) => new Device({
      id: e.id as string,
      token: '', // children don't expose their tokens
      name: e.name as string,
      scopes: (e.scopes || []) as string[],
      url: this.url,
      authUrl: this.authUrl,
    }))
  }
}
