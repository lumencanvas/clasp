/**
 * Parse a duration string like '1h', '30s', '5m', '7d' to milliseconds.
 * Supports: ms, s, m, h, d
 *
 * @throws Error on invalid format, empty string, or negative values
 */
export function parseDuration(s: string): number {
  if (!s || !s.trim()) throw new Error('Duration string must not be empty')
  const match = s.match(/^(\d+(?:\.\d+)?)\s*(ms|s|m|h|d)$/)
  if (!match) throw new Error(`Invalid duration: "${s}". Use format like "30s", "5m", "1h", "7d".`)
  const n = parseFloat(match[1])
  switch (match[2]) {
    case 'ms': return n
    case 's': return n * 1000
    case 'm': return n * 60_000
    case 'h': return n * 3_600_000
    case 'd': return n * 86_400_000
    default: return n * 1000
  }
}

/**
 * Parse a duration string to seconds (float).
 */
export function parseDurationToSeconds(s: string): number {
  return parseDuration(s) / 1000
}

/**
 * Parse a duration string to whole seconds (rounded).
 */
export function parseDurationToWholeSeconds(s: string): number {
  return Math.round(parseDurationToSeconds(s))
}
