import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { execFileSync } from 'child_process'
import { accessSync } from 'fs'

vi.mock('child_process', () => ({
  execFileSync: vi.fn(),
}))

vi.mock('fs', () => ({
  accessSync: vi.fn(),
  constants: { X_OK: 1 },
}))

// Must import after mocks are set up
const { resolveBinary } = await import('../src/binary')

describe('resolveBinary', () => {
  const originalEnv = process.env

  beforeEach(() => {
    vi.resetAllMocks()
    process.env = { ...originalEnv }
    delete process.env.CLASP_RELAY_BIN
  })

  afterEach(() => {
    process.env = originalEnv
  })

  it('returns explicit path when provided and executable', () => {
    vi.mocked(accessSync).mockImplementation(() => {})
    expect(resolveBinary('/usr/local/bin/clasp-relay')).toBe('/usr/local/bin/clasp-relay')
  })

  it('throws when explicit path is not executable', () => {
    vi.mocked(accessSync).mockImplementation(() => { throw new Error('EACCES') })
    expect(() => resolveBinary('/bad/path')).toThrow('not executable')
  })

  it('uses CLASP_RELAY_BIN env var', () => {
    process.env.CLASP_RELAY_BIN = '/env/clasp-relay'
    vi.mocked(accessSync).mockImplementation(() => {})
    expect(resolveBinary()).toBe('/env/clasp-relay')
  })

  it('throws when CLASP_RELAY_BIN path is not executable', () => {
    process.env.CLASP_RELAY_BIN = '/bad/env/path'
    vi.mocked(accessSync).mockImplementation(() => { throw new Error('EACCES') })
    expect(() => resolveBinary()).toThrow('CLASP_RELAY_BIN')
  })

  it('falls back to PATH lookup via which', () => {
    vi.mocked(accessSync).mockImplementation(() => { throw new Error('EACCES') })
    vi.mocked(execFileSync).mockReturnValue('/usr/bin/clasp-relay\n')
    expect(resolveBinary()).toBe('/usr/bin/clasp-relay')
  })

  it('throws helpful error when binary not found anywhere', () => {
    vi.mocked(accessSync).mockImplementation(() => { throw new Error('EACCES') })
    vi.mocked(execFileSync).mockImplementation(() => { throw new Error('not found') })
    expect(() => resolveBinary()).toThrow('Could not find clasp-relay')
    expect(() => resolveBinary()).toThrow('cargo install')
  })

  it('explicit path takes priority over env var', () => {
    process.env.CLASP_RELAY_BIN = '/env/clasp-relay'
    vi.mocked(accessSync).mockImplementation(() => {})
    expect(resolveBinary('/explicit/clasp-relay')).toBe('/explicit/clasp-relay')
  })

  it('env var takes priority over PATH', () => {
    process.env.CLASP_RELAY_BIN = '/env/clasp-relay'
    vi.mocked(accessSync).mockImplementation(() => {})
    vi.mocked(execFileSync).mockReturnValue('/usr/bin/clasp-relay\n')
    expect(resolveBinary()).toBe('/env/clasp-relay')
  })
})
