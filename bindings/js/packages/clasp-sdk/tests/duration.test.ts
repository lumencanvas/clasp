import { describe, it, expect } from 'vitest'
import { parseDuration, parseDurationToSeconds, parseDurationToWholeSeconds } from '../src/duration'

describe('parseDuration', () => {
  it('parses milliseconds', () => {
    expect(parseDuration('100ms')).toBe(100)
  })

  it('parses seconds', () => {
    expect(parseDuration('30s')).toBe(30_000)
  })

  it('parses minutes', () => {
    expect(parseDuration('5m')).toBe(300_000)
  })

  it('parses hours', () => {
    expect(parseDuration('1h')).toBe(3_600_000)
  })

  it('parses days', () => {
    expect(parseDuration('7d')).toBe(604_800_000)
  })

  it('parses decimals (1.5h)', () => {
    expect(parseDuration('1.5h')).toBe(5_400_000)
  })

  it('parses decimals (0.5s)', () => {
    expect(parseDuration('0.5s')).toBe(500)
  })

  it('handles zero values (0s)', () => {
    expect(parseDuration('0s')).toBe(0)
  })

  it('handles zero values (0ms)', () => {
    expect(parseDuration('0ms')).toBe(0)
  })

  it('handles large values (365d)', () => {
    expect(parseDuration('365d')).toBe(365 * 86_400_000)
  })

  it('throws on invalid format (abc)', () => {
    expect(() => parseDuration('abc')).toThrow('Invalid duration')
  })

  it('throws on missing unit (10)', () => {
    expect(() => parseDuration('10')).toThrow('Invalid duration')
  })

  it('throws on unknown unit (10x)', () => {
    expect(() => parseDuration('10x')).toThrow('Invalid duration')
  })

  it('throws on empty string', () => {
    expect(() => parseDuration('')).toThrow('must not be empty')
  })

  it('throws on whitespace-only', () => {
    expect(() => parseDuration('   ')).toThrow('must not be empty')
  })

  it('throws on negative values (-5s)', () => {
    expect(() => parseDuration('-5s')).toThrow('Invalid duration')
  })

  it('is case sensitive (5S throws)', () => {
    expect(() => parseDuration('5S')).toThrow('Invalid duration')
  })

  it('is case sensitive (5M throws)', () => {
    expect(() => parseDuration('5M')).toThrow('Invalid duration')
  })
})

describe('parseDurationToSeconds', () => {
  it('converts 30s to 30', () => {
    expect(parseDurationToSeconds('30s')).toBe(30)
  })

  it('converts 1m to 60', () => {
    expect(parseDurationToSeconds('1m')).toBe(60)
  })

  it('handles sub-second (500ms to 0.5)', () => {
    expect(parseDurationToSeconds('500ms')).toBe(0.5)
  })
})

describe('parseDurationToWholeSeconds', () => {
  it('rounds up (1500ms to 2)', () => {
    expect(parseDurationToWholeSeconds('1500ms')).toBe(2)
  })

  it('rounds down (1499ms to 1)', () => {
    expect(parseDurationToWholeSeconds('1499ms')).toBe(1)
  })

  it('no rounding needed (1000ms to 1)', () => {
    expect(parseDurationToWholeSeconds('1000ms')).toBe(1)
  })
})
