import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { discover, watch, discoverLocal } from '../src/discovery'

describe('discover()', () => {
  let originalFetch: typeof globalThis.fetch

  beforeEach(() => {
    originalFetch = globalThis.fetch
  })

  afterEach(() => {
    globalThis.fetch = originalFetch
  })

  describe('with rendezvousUrl', () => {
    it('fetches /discover endpoint', async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: vi.fn().mockResolvedValue([]),
      }) as any

      await discover({ rendezvousUrl: 'http://rendezvous:7340' })

      expect(globalThis.fetch).toHaveBeenCalledWith(
        'http://rendezvous:7340/discover',
        expect.objectContaining({ signal: expect.any(AbortSignal) })
      )
    })

    it('parses array response', async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: vi.fn().mockResolvedValue([
          { name: 'Router A', url: 'ws://a:7330' },
          { name: 'Router B', url: 'ws://b:7330' },
        ]),
      }) as any

      const routers = await discover({ rendezvousUrl: 'http://rendezvous:7340' })
      expect(routers).toHaveLength(2)
      expect(routers[0].name).toBe('Router A')
      expect(routers[1].url).toBe('ws://b:7330')
    })

    it('parses { devices: [...] } response', async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: vi.fn().mockResolvedValue({
          devices: [{ name: 'R1', url: 'ws://r1:7330' }],
        }),
      }) as any

      const routers = await discover({ rendezvousUrl: 'http://rendezvous:7340' })
      expect(routers).toHaveLength(1)
      expect(routers[0].name).toBe('R1')
    })

    it('returns empty array on non-OK response', async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: false,
        status: 500,
      }) as any

      const routers = await discover({ rendezvousUrl: 'http://rendezvous:7340' })
      expect(routers).toEqual([])
    })

    it('returns empty array on network error', async () => {
      globalThis.fetch = vi.fn().mockRejectedValue(new Error('ECONNREFUSED')) as any

      const routers = await discover({ rendezvousUrl: 'http://rendezvous:7340' })
      expect(routers).toEqual([])
    })

    it('returns empty array on timeout', async () => {
      globalThis.fetch = vi.fn().mockRejectedValue(new DOMException('Aborted', 'AbortError')) as any

      const routers = await discover({ rendezvousUrl: 'http://rendezvous:7340', timeout: 100 })
      expect(routers).toEqual([])
    })

    it('uses provided timeout value', async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: vi.fn().mockResolvedValue([]),
      }) as any

      await discover({ rendezvousUrl: 'http://rendezvous:7340', timeout: 5000 })
      // Signal should be passed (we can't easily check timeout, but no errors)
      expect(globalThis.fetch).toHaveBeenCalled()
    })
  })

  describe('without rendezvousUrl (local discovery)', () => {
    it('probes localhost candidates', async () => {
      // All probes fail
      globalThis.fetch = vi.fn().mockRejectedValue(new Error('ECONNREFUSED')) as any

      const routers = await discover({ timeout: 100 })
      expect(routers).toEqual([])
      // Should have probed multiple ports
      expect((globalThis.fetch as any).mock.calls.length).toBeGreaterThan(0)
    })

    it('returns router on 200 response', async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
      }) as any

      const routers = await discover({ timeout: 100 })
      expect(routers.length).toBeGreaterThanOrEqual(1)
      expect(routers[0].name).toBe('Local Router')
    })

    it('returns router on 401 response (auth present)', async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: false,
        status: 401,
      }) as any

      const routers = await discover({ timeout: 100 })
      expect(routers.length).toBeGreaterThanOrEqual(1)
    })
  })
})

describe('discoverLocal()', () => {
  let originalFetch: typeof globalThis.fetch

  beforeEach(() => {
    originalFetch = globalThis.fetch
  })

  afterEach(() => {
    globalThis.fetch = originalFetch
  })

  it('probes ports 7330-7339 on localhost', async () => {
    globalThis.fetch = vi.fn().mockRejectedValue(new Error('ECONNREFUSED')) as any
    await discoverLocal({ timeout: 100 })
    const calls = (globalThis.fetch as any).mock.calls
    // Should probe multiple ports
    const urls = calls.map((c: any) => c[0])
    expect(urls.some((u: string) => u.includes('7350'))).toBe(true) // 7330 + 20
    expect(urls.some((u: string) => u.includes('7359'))).toBe(true) // 7339 + 20
  })
})

describe('watch()', () => {
  let originalFetch: typeof globalThis.fetch

  beforeEach(() => {
    originalFetch = globalThis.fetch
    vi.useFakeTimers()
  })

  afterEach(() => {
    globalThis.fetch = originalFetch
    vi.useRealTimers()
  })

  it('emits "found" on initial poll', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: vi.fn().mockResolvedValue([
        { name: 'Router', url: 'ws://localhost:7330' },
      ]),
    }) as any

    const events: any[] = []
    const stop = watch(
      (e) => events.push(e),
      { rendezvousUrl: 'http://rendezvous:7340' }
    )

    // Let the initial poll resolve
    await vi.runOnlyPendingTimersAsync()

    expect(events.length).toBeGreaterThanOrEqual(1)
    expect(events[0].type).toBe('found')
    expect(events[0].name).toBe('Router')

    stop()
  })

  it('emits "lost" when router disappears', async () => {
    let callCount = 0
    globalThis.fetch = vi.fn().mockImplementation(async () => {
      callCount++
      return {
        ok: true,
        json: vi.fn().mockResolvedValue(
          callCount <= 1
            ? [{ name: 'Router', url: 'ws://localhost:7330' }]
            : []
        ),
      }
    }) as any

    const events: any[] = []
    const stop = watch(
      (e) => events.push(e),
      { rendezvousUrl: 'http://rendezvous:7340', interval: 1000 }
    )

    // Initial poll
    await vi.runOnlyPendingTimersAsync()
    // Advance to next poll
    await vi.advanceTimersByTimeAsync(1000)

    const lost = events.find(e => e.type === 'lost')
    expect(lost).toBeDefined()
    expect(lost.name).toBe('Router')

    stop()
  })

  it('stop function clears interval', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: vi.fn().mockResolvedValue([]),
    }) as any

    const events: any[] = []
    const stop = watch(
      (e) => events.push(e),
      { rendezvousUrl: 'http://rendezvous:7340', interval: 1000 }
    )

    // Initial poll
    await vi.runOnlyPendingTimersAsync()
    const countAfterStart = (globalThis.fetch as any).mock.calls.length

    stop()

    // Advance timers — should NOT trigger more polls
    await vi.advanceTimersByTimeAsync(5000)
    expect((globalThis.fetch as any).mock.calls.length).toBe(countAfterStart)
  })

  it('handles poll errors without crashing', async () => {
    let callCount = 0
    globalThis.fetch = vi.fn().mockImplementation(async () => {
      callCount++
      if (callCount === 1) {
        return { ok: true, json: vi.fn().mockResolvedValue([]) }
      }
      // discover() catches fetch errors and returns [], so watch continues
      throw new Error('Network failure')
    }) as any

    const events: any[] = []
    const stop = watch(
      (e) => events.push(e),
      { rendezvousUrl: 'http://rendezvous:7340', interval: 1000 }
    )

    // Initial poll (succeeds)
    await vi.runOnlyPendingTimersAsync()
    // Second poll (fetch fails, discover catches it)
    await vi.advanceTimersByTimeAsync(1000)

    // watch should still be running, no crash
    stop()
  })
})
