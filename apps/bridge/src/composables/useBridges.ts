import { ref, readonly } from 'vue'
import type { DirectLink, AnyProtocol } from '../lib/types'
import { useElectron } from './useElectron'
import { loadFromStorage, saveToStorage } from './useStorage'
import { useNotifications } from './useNotifications'
import { defaultAddresses } from '../lib/constants'

const bridges = ref<DirectLink[]>([])
const editingBridge = ref<DirectLink | null>(null)

function saveBridges() {
  saveToStorage('bridges', bridges.value)
}

async function restore() {
  const saved = loadFromStorage<Partial<DirectLink>[]>('bridges', [])
  const { invoke } = useElectron()
  bridges.value = []

  for (const config of saved) {
    const link = { ...config, active: false } as DirectLink
    try {
      const result = await invoke<DirectLink>('createBridge', config)
      link.id = result?.id || link.id
      link.active = true
    } catch {
      link.active = false
    }
    bridges.value.push(link)
  }
}

async function add(config: { source: AnyProtocol; sourceAddr: string; target: AnyProtocol; targetAddr: string }) {
  const { invoke } = useElectron()
  const { notify } = useNotifications()

  const existing = bridges.value.find(b =>
    b.source === config.source && b.target === config.target &&
    b.sourceAddr === config.sourceAddr && b.targetAddr === config.targetAddr
  )
  if (existing) return

  try {
    let bridge: DirectLink
    const result = await invoke<DirectLink>('createBridge', config)
    bridge = result || { id: Date.now().toString(), ...config, active: true }
    bridges.value.push(bridge)
    saveBridges()
  } catch (err: any) {
    notify(`Failed to create bridge: ${err.message}`, 'error')
  }
}

async function remove(id: string) {
  const { invoke } = useElectron()
  try {
    await invoke('deleteBridge', id)
  } catch (err) {
    console.error('Failed to delete bridge:', err)
  }
  bridges.value = bridges.value.filter(b => b.id !== id)
  saveBridges()
}

function edit(id: string) {
  editingBridge.value = bridges.value.find(b => b.id === id) || null
}

function cancelEdit() {
  editingBridge.value = null
}

function handleBridgeEvent(data: { bridgeId: string; event: string; data?: string }) {
  const { notify } = useNotifications()
  const bridge = bridges.value.find(b => b.id === data.bridgeId)
  if (!bridge) return

  if (data.event === 'connected') {
    bridge.active = true
    notify('Bridge connected', 'success')
  } else if (data.event === 'disconnected') {
    bridge.active = false
    if (data.data) notify(`Bridge disconnected: ${data.data}`, 'warning')
  } else if (data.event === 'error') {
    notify(`Bridge error: ${data.data}`, 'error')
  }
}

export function useBridges() {
  return {
    bridges: readonly(bridges),
    editingBridge: readonly(editingBridge),
    restore,
    add,
    remove,
    edit,
    cancelEdit,
    saveBridges,
    handleBridgeEvent,
  }
}
