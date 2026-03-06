import { ref, readonly, computed } from 'vue'
import type { Signal, SignalHistory } from '../lib/types'

const signals = ref<Signal[]>([])
const paused = ref(false)
const maxSignals = ref(200)
const monitorFilter = ref('')
const protocolFilter = ref('all')
const signalRate = ref(0)
const signalHistory = ref(new Map<string, SignalHistory>())

let signalCount = 0
let rateInterval: ReturnType<typeof setInterval> | null = null

function startRateCounter() {
  if (rateInterval) clearInterval(rateInterval)
  rateInterval = setInterval(() => {
    signalRate.value = signalCount
    signalCount = 0
  }, 1000)
}

function stopRateCounter() {
  if (rateInterval) {
    clearInterval(rateInterval)
    rateInterval = null
  }
}

function addSignal(signal: Signal) {
  if (paused.value) return

  signalCount++
  const enriched: Signal = {
    ...signal,
    timestamp: Date.now(),
    protocol: signal.protocol || detectProtocol(signal),
  }

  signals.value.unshift(enriched)
  if (signals.value.length > maxSignals.value) {
    signals.value = signals.value.slice(0, maxSignals.value)
  }

  // Track history for sparklines
  const address = signal.address || signal.bridgeId || 'unknown'
  const value = typeof signal.value === 'number' ? signal.value :
    signal.velocity !== undefined ? signal.velocity / 127 : 0

  if (!signalHistory.value.has(address)) {
    signalHistory.value.set(address, { values: [], updateCount: 0, lastUpdate: Date.now() })
  }
  const history = signalHistory.value.get(address)!
  history.values.push(value)
  history.updateCount++
  history.lastUpdate = Date.now()
  if (history.values.length > 50) history.values.shift()

  // Periodic cleanup
  if (history.updateCount % 100 === 0) cleanupHistory()
}

function detectProtocol(signal: Signal): string {
  if (signal.address?.startsWith('/mqtt/')) return 'mqtt'
  if (signal.address?.startsWith('/osc/')) return 'osc'
  if (signal.address?.startsWith('/')) return 'clasp'
  if (signal.topic !== undefined) return 'mqtt'
  if (signal.channel !== undefined && (signal.note !== undefined || signal.cc !== undefined)) return 'midi'
  if (signal.universe !== undefined) return 'artnet'
  return 'clasp'
}

function cleanupHistory() {
  const staleThreshold = 5 * 60 * 1000
  const now = Date.now()
  for (const [addr, h] of signalHistory.value) {
    if (now - h.lastUpdate > staleThreshold) signalHistory.value.delete(addr)
  }
  if (signalHistory.value.size > 500) {
    const entries = [...signalHistory.value.entries()].sort((a, b) => a[1].lastUpdate - b[1].lastUpdate)
    for (const [addr] of entries.slice(0, entries.length - 500)) {
      signalHistory.value.delete(addr)
    }
  }
}

function togglePause() {
  paused.value = !paused.value
}

function clear() {
  signals.value = []
}

function setMaxSignals(max: number) {
  maxSignals.value = max
  if (signals.value.length > max) signals.value = signals.value.slice(0, max)
}

function setFilter(filter: string) {
  monitorFilter.value = filter.toLowerCase()
}

function setProtocolFilter(filter: string) {
  protocolFilter.value = filter
}

const filteredSignals = computed(() => {
  let result = signals.value
  if (protocolFilter.value && protocolFilter.value !== 'all') {
    result = result.filter(s => s.protocol === protocolFilter.value)
  }
  if (monitorFilter.value) {
    result = result.filter(s =>
      (s.address && s.address.toLowerCase().includes(monitorFilter.value)) ||
      (s.bridgeId && s.bridgeId.toLowerCase().includes(monitorFilter.value)) ||
      (s.serverName && s.serverName.toLowerCase().includes(monitorFilter.value))
    )
  }
  return result.slice(0, 100)
})

export function useMonitor() {
  return {
    signals: readonly(signals),
    filteredSignals,
    paused: readonly(paused),
    maxSignals: readonly(maxSignals),
    monitorFilter: readonly(monitorFilter),
    protocolFilter: readonly(protocolFilter),
    signalRate: readonly(signalRate),
    signalHistory: readonly(signalHistory),
    startRateCounter,
    stopRateCounter,
    addSignal,
    togglePause,
    clear,
    setMaxSignals,
    setFilter,
    setProtocolFilter,
  }
}
