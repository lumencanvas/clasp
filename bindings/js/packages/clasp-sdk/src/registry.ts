/**
 * Entity registry client for CLASP relay REST API.
 */
export interface Entity {
  id: string
  name: string
  type: 'device' | 'user' | 'service' | 'router'
  status?: string
  public_key?: string
  created_at?: string
}

export interface RegistryClientOptions {
  /** Relay auth URL (e.g., https://relay.example.com) */
  url: string
  /** Admin token for authentication */
  token: string
}

export class RegistryClient {
  private url: string
  private token: string

  constructor(options: RegistryClientOptions) {
    this.url = options.url.replace(/\/+$/, '')
    this.token = options.token
  }

  private async request(path: string, options: RequestInit = {}): Promise<any> {
    const resp = await fetch(`${this.url}${path}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.token}`,
        ...options.headers,
      },
    })
    if (!resp.ok) {
      const text = await resp.text()
      throw new Error(`${resp.status} ${resp.statusText}: ${text}`)
    }
    return resp.json()
  }

  async list(): Promise<Entity[]> {
    return this.request('/api/entities')
  }

  async get(id: string): Promise<Entity> {
    return this.request(`/api/entities/${encodeURIComponent(id)}`)
  }

  async create(name: string, type: Entity['type'] = 'device'): Promise<Entity> {
    return this.request('/api/entities', {
      method: 'POST',
      body: JSON.stringify({ name, type }),
    })
  }

  async updateStatus(id: string, status: string): Promise<Entity> {
    return this.request(`/api/entities/${encodeURIComponent(id)}/status`, {
      method: 'PUT',
      body: JSON.stringify({ status }),
    })
  }

  async delete(id: string): Promise<void> {
    await this.request(`/api/entities/${encodeURIComponent(id)}`, {
      method: 'DELETE',
    })
  }
}
