/**
 * Journal query client for CLASP relay REST API.
 */
export interface JournalEntry {
  seq: number
  timestamp: number
  author: string
  address: string
  signal_type: string
  value: any
  revision?: number
  msg_type: number
}

export interface ParamSnapshot {
  address: string
  value: any
  revision: number
  writer: string
  timestamp: number
}

export interface JournalQueryOptions {
  pattern?: string
  from?: number
  to?: number
  limit?: number
  types?: string[]
}

export interface JournalClientOptions {
  url: string
  token: string
}

export class JournalClient {
  private url: string
  private token: string

  constructor(options: JournalClientOptions) {
    this.url = options.url.replace(/\/+$/, '')
    this.token = options.token
  }

  private headers(): Record<string, string> {
    return {
      'Authorization': `Bearer ${this.token}`,
      'Content-Type': 'application/json',
    }
  }

  async query(options: JournalQueryOptions = {}): Promise<JournalEntry[]> {
    const params = new URLSearchParams()
    if (options.pattern) params.set('pattern', options.pattern)
    if (options.from != null) params.set('from', String(options.from))
    if (options.to != null) params.set('to', String(options.to))
    if (options.limit != null) params.set('limit', String(options.limit))
    if (options.types?.length) params.set('types', options.types.join(','))

    const resp = await fetch(`${this.url}/api/journal/query?${params}`, {
      headers: this.headers(),
    })
    if (!resp.ok) throw new Error(`Journal query failed: ${resp.status}`)
    return resp.json()
  }

  async since(seq: number, limit?: number): Promise<JournalEntry[]> {
    const params = new URLSearchParams({ seq: String(seq) })
    if (limit != null) params.set('limit', String(limit))

    const resp = await fetch(`${this.url}/api/journal/since?${params}`, {
      headers: this.headers(),
    })
    if (!resp.ok) throw new Error(`Journal since failed: ${resp.status}`)
    return resp.json()
  }

  async latestSeq(): Promise<number> {
    const resp = await fetch(`${this.url}/api/journal/latest`, {
      headers: this.headers(),
    })
    if (!resp.ok) throw new Error(`Journal latest failed: ${resp.status}`)
    const data = await resp.json()
    return data.seq
  }

  async loadSnapshot(): Promise<ParamSnapshot[]> {
    const resp = await fetch(`${this.url}/api/journal/snapshot`, {
      headers: this.headers(),
    })
    if (!resp.ok) throw new Error(`Journal snapshot failed: ${resp.status}`)
    return resp.json()
  }
}
