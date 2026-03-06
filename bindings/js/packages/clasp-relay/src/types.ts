import type { RelayConfig } from '@clasp-to/sdk'

export type { RelayConfig }

export interface RelayServerOptions {
  /** Path to clasp-relay binary. Auto-detected if not set. */
  binary?: string
  /** Working directory for the relay process. */
  cwd?: string
  /** Environment variables to pass to the process. */
  env?: Record<string, string>
  /** Timeout (ms) for readiness probe. Default: 10000 */
  readyTimeout?: number
  /** If true, pipe relay stdout/stderr to parent process. Default: false */
  inherit?: boolean
}

export interface RelayEvents {
  log: (line: string) => void
  error: (err: Error) => void
  exit: (code: number | null, signal: string | null) => void
  ready: () => void
}

export type ConfigInput = Partial<RelayConfig> | ((b: any) => any)
