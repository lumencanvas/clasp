const cache = new Map<string, RegExp>()
const MAX_CACHE = 1000
const accessOrder: string[] = []

export function compilePattern(pattern: string): RegExp {
  const cached = cache.get(pattern)
  if (cached) {
    // Move to end of access order
    const idx = accessOrder.indexOf(pattern)
    if (idx !== -1) accessOrder.splice(idx, 1)
    accessOrder.push(pattern)
    return cached
  }

  // Compile: ** matches any path segments, * matches single segment
  const regexStr = pattern
    .replace(/[.+^${}()|[\]\\]/g, '\\$&') // escape regex special chars (except *)
    .replace(/\*\*/g, '\u0000')  // temp placeholder for **
    .replace(/\*/g, '[^/]+')      // * = single segment
    .replace(/\u0000/g, '.*')     // ** = any segments

  const regex = new RegExp(`^${regexStr}$`)

  // Evict if at capacity
  if (cache.size >= MAX_CACHE) {
    const oldest = accessOrder.shift()
    if (oldest) cache.delete(oldest)
  }

  cache.set(pattern, regex)
  accessOrder.push(pattern)
  return regex
}

export function matchPattern(pattern: string, address: string): boolean {
  return compilePattern(pattern).test(address)
}

export function clearPatternCache(): void {
  cache.clear()
  accessOrder.length = 0
}
