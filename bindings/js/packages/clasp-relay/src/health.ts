import { request } from 'http'
import { createConnection } from 'net'

/**
 * Wait for an HTTP endpoint to respond (for health-port or auth-port).
 */
export function waitForHttp(
  port: number,
  host: string,
  timeout: number,
  path = '/healthz',
): Promise<void> {
  return new Promise((resolve, reject) => {
    const start = Date.now()
    const interval = 100

    function poll() {
      if (Date.now() - start > timeout) {
        reject(new Error(`Relay HTTP endpoint did not respond within ${timeout}ms (port ${port})`))
        return
      }

      const req = request(
        { host, port, path, method: 'GET', timeout: 500 },
        (res) => {
          res.resume()
          resolve()
        },
      )

      req.on('error', () => {
        setTimeout(poll, interval)
      })

      req.on('timeout', () => {
        req.destroy()
        setTimeout(poll, interval)
      })

      req.end()
    }

    poll()
  })
}

/**
 * Wait for a TCP port to accept connections (for WS-only readiness).
 */
export function waitForTcp(
  port: number,
  host: string,
  timeout: number,
): Promise<void> {
  return new Promise((resolve, reject) => {
    const start = Date.now()
    const interval = 100

    function poll() {
      if (Date.now() - start > timeout) {
        reject(new Error(`Relay TCP port did not accept connections within ${timeout}ms (port ${port})`))
        return
      }

      const socket = createConnection({ host, port }, () => {
        socket.destroy()
        resolve()
      })

      socket.on('error', () => {
        socket.destroy()
        setTimeout(poll, interval)
      })

      socket.setTimeout(500, () => {
        socket.destroy()
        setTimeout(poll, interval)
      })
    }

    poll()
  })
}

/**
 * Wait for the relay to become ready, using the best available probe:
 * 1. Health port (HTTP /healthz) if configured
 * 2. Auth port (HTTP) if configured
 * 3. TCP connect to WS port as last resort
 */
export function waitForReady(
  wsPort: number,
  healthPort: number | undefined,
  authPort: number | undefined,
  host: string,
  timeout: number,
): Promise<void> {
  if (healthPort) {
    return waitForHttp(healthPort, host, timeout, '/healthz')
  }
  if (authPort) {
    return waitForHttp(authPort, host, timeout, '/')
  }
  return waitForTcp(wsPort, host, timeout)
}

/**
 * Parse a port number from relay startup output.
 * Looks for patterns like "listening on 0.0.0.0:7330" or "ws port: 7330"
 * or the actual relay log format "WebSocket server listening on 0.0.0.0:7330".
 */
export function parsePortFromOutput(line: string): number | null {
  // "WebSocket server listening on 0.0.0.0:7330"
  // "listening on 0.0.0.0:7330" or "listening on [::]:7330"
  const listenMatch = line.match(/listening on [\w.:[\]]+:(\d+)/)
  if (listenMatch) return parseInt(listenMatch[1], 10)

  // "ws port: 7330" or "WebSocket port: 7330"
  const portMatch = line.match(/(?:ws|websocket)\s*port[:\s]+(\d+)/i)
  if (portMatch) return parseInt(portMatch[1], 10)

  // "WebSocket: ws://0.0.0.0:7330" (relay banner format)
  const wsUrlMatch = line.match(/WebSocket:\s*wss?:\/\/[\w.:[\]]+:(\d+)/)
  if (wsUrlMatch) return parseInt(wsUrlMatch[1], 10)

  return null
}
