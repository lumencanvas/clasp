import { execFileSync } from 'child_process'
import { accessSync, constants } from 'fs'
import { join } from 'path'

const BINARY_NAME = 'clasp-relay'

const PLATFORM_PACKAGES: Record<string, string> = {
  'darwin-arm64': '@clasp-to/relay-darwin-arm64',
  'darwin-x64': '@clasp-to/relay-darwin-x64',
  'linux-x64': '@clasp-to/relay-linux-x64',
  'linux-arm64': '@clasp-to/relay-linux-arm64',
}

function isExecutable(path: string): boolean {
  try {
    accessSync(path, constants.X_OK)
    return true
  } catch {
    return false
  }
}

function tryPlatformPackage(): string | null {
  const key = `${process.platform}-${process.arch}`
  const pkg = PLATFORM_PACKAGES[key]
  if (!pkg) return null

  try {
    const pkgPath = require.resolve(`${pkg}/package.json`)
    const binPath = join(pkgPath, '..', 'bin', BINARY_NAME)
    if (isExecutable(binPath)) return binPath
  } catch {
    // Package not installed
  }
  return null
}

function tryWhich(): string | null {
  try {
    const result = execFileSync('which', [BINARY_NAME], {
      encoding: 'utf-8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim()
    return result || null
  } catch {
    return null
  }
}

/**
 * Resolve the clasp-relay binary path.
 *
 * Resolution order:
 * 1. Explicit path (passed as argument)
 * 2. CLASP_RELAY_BIN environment variable
 * 3. Platform-specific npm package (@clasp-to/relay-<platform>)
 * 4. PATH lookup (which clasp-relay)
 */
export function resolveBinary(explicit?: string): string {
  // 1. Explicit path
  if (explicit) {
    if (!isExecutable(explicit)) {
      throw new Error(`clasp-relay binary not executable at: ${explicit}`)
    }
    return explicit
  }

  // 2. Environment variable
  const envPath = process.env.CLASP_RELAY_BIN
  if (envPath) {
    if (!isExecutable(envPath)) {
      throw new Error(
        `CLASP_RELAY_BIN is set to "${envPath}" but the file is not executable`
      )
    }
    return envPath
  }

  // 3. Platform package
  const pkgPath = tryPlatformPackage()
  if (pkgPath) return pkgPath

  // 4. PATH lookup
  const whichPath = tryWhich()
  if (whichPath) return whichPath

  throw new Error(
    `Could not find clasp-relay binary. Install it via one of:\n` +
    `  - npm install @clasp-to/relay-${process.platform}-${process.arch}\n` +
    `  - cargo install clasp-relay\n` +
    `  - Set CLASP_RELAY_BIN environment variable\n` +
    `  - Add clasp-relay to your PATH`
  )
}
