<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import AppShell from './components/layout/AppShell.vue'
import ToastContainer from './components/shared/ToastContainer.vue'
import OnboardingModal from './components/modals/OnboardingModal.vue'
import { useElectron } from './composables/useElectron'
import { useRouters } from './composables/useRouters'
import { useConnections } from './composables/useConnections'
import { useBridges } from './composables/useBridges'
import { useRoutes } from './composables/useRoutes'
import { useMonitor } from './composables/useMonitor'
import { useDevices } from './composables/useDevices'
import { useTokens } from './composables/useTokens'
import { useLogs } from './composables/useLogs'
import { useDiagnostics } from './composables/useDiagnostics'
import { useLearnMode } from './composables/useLearnMode'
import { useOnboarding } from './composables/useOnboarding'
import { useNotifications } from './composables/useNotifications'

const { on } = useElectron()
const { restore: restoreRouters, handleStatusUpdate: handleRouterStatus } = useRouters()
const { restore: restoreConnections, handleStatusUpdate: handleConnStatus, handleRouterStatus: handleBridgeRouterStatus } = useConnections()
const { restore: restoreBridges, handleBridgeEvent } = useBridges()
const { load: loadRoutes, processSignal } = useRoutes()
const { addSignal, startRateCounter, stopRateCounter } = useMonitor()
const { load: loadDevices, upsert: upsertDevice } = useDevices()
const { load: loadTokens } = useTokens()
const { addServerLog } = useLogs()
const { checkBridgeStatus, updateStats, setBridgeReady } = useDiagnostics()
const { handleSignal: handleLearnSignal } = useLearnMode()
const { checkFirstRun } = useOnboarding()
const { notify } = useNotifications()

const cleanups: Array<() => void> = []

onMounted(async () => {
  // Load persisted data
  loadRoutes()
  loadTokens()

  // Wait for bridge service, then restore state
  await checkBridgeStatus()
  await restoreRouters()
  await restoreConnections()
  await restoreBridges()
  await loadDevices()

  // Start signal rate counter
  startRateCounter()

  // Set up IPC event listeners
  cleanups.push(on('deviceFound', (device) => {
    upsertDevice(device)
  }))
  cleanups.push(on('signal', (signal) => {
    if (handleLearnSignal(signal)) return
    addSignal(signal)
    processSignal(signal)
  }))
  cleanups.push(on('serverStatus', (status) => {
    if (!handleRouterStatus(status)) {
      handleConnStatus(status)
    }
  }))
  cleanups.push(on('serverLog', (data) => {
    addServerLog(data.serverId, data.log)
  }))
  cleanups.push(on('bridgeEvent', (data) => {
    handleBridgeEvent(data)
  }))
  cleanups.push(on('serverStatsUpdate', (stats) => {
    updateStats(stats)
  }))
  cleanups.push(on('bridgeReady', (ready) => {
    setBridgeReady(ready)
    notify(ready ? 'Bridge service connected' : 'Bridge service disconnected', ready ? 'success' : 'error')
  }))
  cleanups.push(on('bridgeRouterStatus', (status) => {
    handleBridgeRouterStatus(status)
  }))

  // Check first run
  checkFirstRun()
})

onUnmounted(() => {
  stopRateCounter()
  cleanups.forEach(fn => fn())
})
</script>

<template>
  <AppShell />
  <ToastContainer />
  <OnboardingModal />
</template>
