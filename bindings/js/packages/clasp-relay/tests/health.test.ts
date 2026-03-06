import { describe, it, expect } from 'vitest'
import { parsePortFromOutput } from '../src/health'

describe('parsePortFromOutput', () => {
  it('parses "listening on 0.0.0.0:7330"', () => {
    expect(parsePortFromOutput('listening on 0.0.0.0:7330')).toBe(7330)
  })

  it('parses "listening on [::]:8080"', () => {
    expect(parsePortFromOutput('listening on [::]:8080')).toBe(8080)
  })

  it('parses "listening on 127.0.0.1:9000"', () => {
    expect(parsePortFromOutput('listening on 127.0.0.1:9000')).toBe(9000)
  })

  it('parses "WebSocket server listening on 0.0.0.0:7330" (actual relay format)', () => {
    expect(parsePortFromOutput('WebSocket server listening on 0.0.0.0:7330')).toBe(7330)
  })

  it('parses "WebSocket: ws://0.0.0.0:7330" (relay banner format)', () => {
    expect(parsePortFromOutput('WebSocket: ws://0.0.0.0:7330')).toBe(7330)
  })

  it('parses "WebSocket: wss://0.0.0.0:7331" (TLS variant)', () => {
    expect(parsePortFromOutput('WebSocket: wss://0.0.0.0:7331')).toBe(7331)
  })

  it('parses "ws port: 7330"', () => {
    expect(parsePortFromOutput('ws port: 7330')).toBe(7330)
  })

  it('parses "WebSocket port: 4000"', () => {
    expect(parsePortFromOutput('WebSocket port: 4000')).toBe(4000)
  })

  it('returns null for unrelated output', () => {
    expect(parsePortFromOutput('relay started successfully')).toBeNull()
  })

  it('returns null for empty string', () => {
    expect(parsePortFromOutput('')).toBeNull()
  })

  it('returns null for non-WS port mentions', () => {
    expect(parsePortFromOutput('MQTT: mqtt://0.0.0.0:1883')).toBeNull()
  })
})
