import { ref, readonly } from 'vue'
import { useElectron } from './useElectron'
import { useRouters } from './useRouters'
import { useConnections } from './useConnections'
import { useBridges } from './useBridges'
import { useNotifications } from './useNotifications'
import { getPreset } from '../presets/index.js'

const step = ref(1)
const selectedUseCase = ref<string | null>(null)
const visible = ref(false)

function nextStep() {
  step.value++
}

function prevStep() {
  step.value--
}

function selectUseCase(useCase: string) {
  selectedUseCase.value = useCase
}

function goToStep(s: number) {
  step.value = s
}

async function checkFirstRun() {
  const { invoke } = useElectron()
  try {
    const isFirst = await invoke<boolean>('isFirstRun')
    if (isFirst) visible.value = true
  } catch { /* ignore */ }
}

async function applyPreset(presetId: string) {
  const preset = getPreset(presetId)
  if (!preset) return

  const { notify } = useNotifications()
  const { add: addRouter } = useRouters()
  const { add: addConnection } = useConnections()
  const { add: addBridge } = useBridges()

  // First server is always the CLASP router
  const routerConfigs = preset.servers.filter((s: any) => s.type === 'clasp')
  const connectionConfigs = preset.servers.filter((s: any) => s.type !== 'clasp')

  // Create routers
  for (const config of routerConfigs) {
    await addRouter({
      type: config.type,
      protocol: config.type,
      name: config.name,
      address: config.address,
      announce: config.announce,
    })
  }

  // Create connections
  for (const config of connectionConfigs) {
    await addConnection({
      type: config.type as any,
      protocol: config.type as any,
      name: config.name,
      address: config.bind ? `${config.bind}:${config.port}` : config.address,
      bind: config.bind,
      port: config.port,
      host: config.host,
      topics: config.topics,
      mode: config.mode,
      basePath: config.basePath,
      cors: config.cors,
      subnet: config.subnet,
      universe: config.universe,
    })
  }

  // Create bridges (direct links)
  for (const config of preset.bridges) {
    await addBridge({
      source: config.source,
      sourceAddr: config.sourceAddr,
      target: config.target,
      targetAddr: config.targetAddr,
    })
  }

  notify(`Applied "${preset.name}" preset`, 'success')
}

async function finish() {
  visible.value = false
  const { invoke } = useElectron()
  try {
    await invoke('setFirstRunComplete')
  } catch { /* ignore */ }
}

function skip() {
  visible.value = false
}

export function useOnboarding() {
  return {
    step: readonly(step),
    selectedUseCase: readonly(selectedUseCase),
    visible: readonly(visible),
    nextStep,
    prevStep,
    selectUseCase,
    goToStep,
    checkFirstRun,
    applyPreset,
    finish,
    skip,
  }
}
