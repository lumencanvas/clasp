import type { DiscoveredRouter, DiscoveryEvent } from './types'

/** Default discovery timeout in milliseconds. */
const DEFAULT_DISCOVERY_TIMEOUT = 3000

/** Default poll interval in milliseconds. */
const DEFAULT_POLL_INTERVAL = 5000

/**
 * Discover CLASP routers on the local network.
 * Uses the rendezvous/discovery endpoint if available.
 *
 * @param options.timeout - Discovery timeout in milliseconds (default: 3000)
 * @param options.rendezvousUrl - Optional rendezvous server URL
 */
export async function discover(options: {
  timeout?: number
  rendezvousUrl?: string
} = {}): Promise<DiscoveredRouter[]> {
  const { timeout = DEFAULT_DISCOVERY_TIMEOUT, rendezvousUrl } = options

  if (rendezvousUrl) {
    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), timeout)
    try {
      const res = await fetch(`${rendezvousUrl}/discover`, {
        signal: controller.signal,
      })
      if (!res.ok) return []
      const data = await res.json()
      const devices = Array.isArray(data) ? data : data.devices || []
      return devices.map((d: Record<string, unknown>) => ({
        name: (d.name as string) || 'Unknown',
        url: (d.url as string) || (d.endpoints as string[])?.[0] || '',
      }))
    } catch {
      return []
    } finally {
      clearTimeout(timer)
    }
  }

  return discoverLocal({ timeout })
}

/**
 * Discover CLASP routers on localhost and common LAN ports.
 * Probes ports 7330-7339 on localhost.
 */
export async function discoverLocal(options: {
  timeout?: number
} = {}): Promise<DiscoveredRouter[]> {
  const { timeout = DEFAULT_DISCOVERY_TIMEOUT } = options

  const candidates: string[] = []
  for (let port = 7330; port <= 7339; port++) {
    candidates.push(`ws://localhost:${port}`)
    candidates.push(`ws://127.0.0.1:${port}`)
  }

  const results: DiscoveredRouter[] = []
  const seen = new Set<string>()

  const checks = candidates.map(async (url) => {
    try {
      const wsPort = parseInt(new URL(url).port, 10)
      const authPort = wsPort + 20
      const hostname = new URL(url).hostname
      const httpUrl = `http://${hostname}:${authPort}`
      const controller = new AbortController()
      const timer = setTimeout(() => controller.abort(), timeout)
      const res = await fetch(`${httpUrl}/auth/guest`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: '{}',
        signal: controller.signal,
      })
      clearTimeout(timer)
      if (res.ok || res.status === 401 || res.status === 403) {
        // Deduplicate localhost and 127.0.0.1
        const key = `${wsPort}`
        if (!seen.has(key)) {
          seen.add(key)
          results.push({ name: 'Local Router', url: `ws://localhost:${wsPort}` })
        }
      }
    } catch {
      // Not reachable
    }
  })

  await Promise.all(checks)
  return results
}

/**
 * Watch for routers appearing and disappearing on the network.
 * Polls the rendezvous server at the given interval.
 *
 * @returns A stop function to cancel watching.
 */
export function watch(
  callback: (event: DiscoveryEvent) => void,
  options: { rendezvousUrl: string; interval?: number; timeout?: number } = { rendezvousUrl: '' }
): () => void {
  const { rendezvousUrl, interval = DEFAULT_POLL_INTERVAL, timeout } = options
  const known = new Map<string, DiscoveredRouter>()
  let stopped = false

  const poll = async () => {
    if (stopped) return
    try {
      const routers = await discover({ rendezvousUrl, timeout })
      const currentUrls = new Set<string>()

      for (const router of routers) {
        currentUrls.add(router.url)
        if (!known.has(router.url)) {
          known.set(router.url, router)
          callback({ type: 'found', name: router.name, url: router.url })
        }
      }

      for (const [url, router] of known) {
        if (!currentUrls.has(url)) {
          known.delete(url)
          callback({ type: 'lost', name: router.name })
        }
      }
    } catch (err) {
      callback({ type: 'error', name: '', error: err instanceof Error ? err : new Error(String(err)) })
    }
  }

  // Initial poll
  poll()
  const timer = setInterval(poll, interval)

  return () => {
    stopped = true
    clearInterval(timer)
  }
}
