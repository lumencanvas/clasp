import { ref, readonly } from 'vue'
import type { ServerStats, DiagnosticsResult } from '../lib/types'
import { useElectron } from './useElectron'

const serverStats = ref(new Map<string, ServerStats>())
const bridgeServiceReady = ref(false)
const diagnosticsResult = ref<DiagnosticsResult | null>(null)
const continuousTestInterval = ref<ReturnType<typeof setInterval> | null>(null)

async function checkBridgeStatus() {
  const { invoke } = useElectron()
  try {
    const status = await invoke<{ ready: boolean }>('getBridgeStatus')
    bridgeServiceReady.value = status?.ready ?? false
  } catch {
    bridgeServiceReady.value = false
  }
}

function updateStats(stats: ServerStats[]) {
  for (const stat of stats) {
    serverStats.value.set(stat.id, stat)
  }
}

function setBridgeReady(ready: boolean) {
  bridgeServiceReady.value = ready
}

async function runDiagnostics() {
  const { invoke } = useElectron()
  diagnosticsResult.value = await invoke<DiagnosticsResult>('runDiagnostics')
  return diagnosticsResult.value
}

async function healthCheck(id: string) {
  const { invoke } = useElectron()
  return invoke<{ healthy: boolean }>('healthCheck', id)
}

async function sendTestSignal(config: { protocol: string; address: string; signalAddress: string; value: any }) {
  const { invoke } = useElectron()
  return invoke<{ success: boolean; error?: string }>('sendTestSignal', config)
}

function startContinuousTest(callback: () => void, intervalMs = 100) {
  stopContinuousTest()
  continuousTestInterval.value = setInterval(callback, intervalMs)
}

function stopContinuousTest() {
  if (continuousTestInterval.value) {
    clearInterval(continuousTestInterval.value)
    continuousTestInterval.value = null
  }
}

function formatUptime(ms: number): string {
  const seconds = Math.floor(ms / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  if (days > 0) return `${days}d ${hours % 24}h`
  if (hours > 0) return `${hours}h ${minutes % 60}m`
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`
  return `${seconds}s`
}

export function useDiagnostics() {
  return {
    serverStats: readonly(serverStats),
    bridgeServiceReady: readonly(bridgeServiceReady),
    diagnosticsResult: readonly(diagnosticsResult),
    continuousTestInterval: readonly(continuousTestInterval),
    checkBridgeStatus,
    updateStats,
    setBridgeReady,
    runDiagnostics,
    healthCheck,
    sendTestSignal,
    startContinuousTest,
    stopContinuousTest,
    formatUptime,
  }
}
