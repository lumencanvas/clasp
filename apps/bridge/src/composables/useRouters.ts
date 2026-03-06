import { ref, readonly, computed } from 'vue'
import type { Router } from '../lib/types'
import { useElectron } from './useElectron'
import { loadFromStorage, saveToStorage } from './useStorage'
import { useNotifications } from './useNotifications'

const routers = ref<Router[]>([])
const editingRouter = ref<Router | null>(null)

// Persistence
function loadRouters(): Router[] {
  const saved = loadFromStorage<Partial<Router>[]>('routers', [])
  return saved.map(r => ({ ...r, status: 'disconnected' } as Router))
}

function saveRouters() {
  const toSave = routers.value.map(r => {
    const { status, error, ...rest } = r
    return rest
  })
  saveToStorage('routers', toSave)
}

async function restore() {
  const saved = loadRouters()
  const { invoke } = useElectron()

  for (const config of saved) {
    if (config.isRemote) {
      config.status = 'available'
      routers.value.push(config)
      continue
    }

    try {
      const result = await invoke<{ id: string }>('startServer', config)
      config.id = result?.id || config.id
      config.status = 'connected'
    } catch (err: any) {
      config.status = 'error'
      config.error = err.message
    }
    routers.value.push(config)
  }
}

async function add(config: Partial<Router>) {
  const { notify } = useNotifications()
  const { invoke } = useElectron()
  const isEditing = editingRouter.value !== null

  const router: Router = {
    id: isEditing ? editingRouter.value!.id : Date.now().toString(),
    type: 'clasp',
    protocol: 'clasp',
    status: 'starting',
    name: config.name || `CLASP Router @ ${config.address || 'localhost:7330'}`,
    address: config.address || 'localhost:7330',
    announce: config.announce,
    authEnabled: config.authEnabled,
    token: config.token,
    tokenFileContent: config.tokenFileContent,
    ...config,
  } as Router

  try {
    const result = await invoke<{ id: string }>('startServer', router)
    router.id = result?.id || router.id
    router.status = 'connected'

    if (isEditing) {
      const idx = routers.value.findIndex(r => r.id === router.id)
      if (idx !== -1) routers.value[idx] = router
      else routers.value.push(router)
    } else {
      routers.value.push(router)
    }
    editingRouter.value = null
    saveRouters()
    return true
  } catch (err: any) {
    router.status = 'error'
    router.error = err.message

    if (isEditing) {
      const idx = routers.value.findIndex(r => r.id === router.id)
      if (idx !== -1) routers.value[idx] = router
      else routers.value.push(router)
    } else {
      routers.value.push(router)
    }
    editingRouter.value = null
    saveRouters()
    notify(`Failed to start router: ${err.message}`, 'error')
    return false
  }
}

async function remove(id: string) {
  const { invoke } = useElectron()
  const { notify } = useNotifications()
  const router = routers.value.find(r => r.id === id)

  if (router && !router.isRemote) {
    try {
      await invoke('stopServer', id)
    } catch (err: any) {
      console.error('Failed to stop router:', err)
    }
  }

  routers.value = routers.value.filter(r => r.id !== id)
  saveRouters()

  if (router?.isRemote) {
    notify(`Removed remote router: ${router.name}`, 'info')
  }
}

async function restart(id: string) {
  const { invoke } = useElectron()
  const { notify } = useNotifications()
  const router = routers.value.find(r => r.id === id)
  if (!router) return

  router.status = 'reconnecting'
  router.error = undefined
  notify(`Restarting ${router.name}...`, 'info')

  try {
    await invoke('stopServer', id)
    await new Promise(resolve => setTimeout(resolve, 500))
    const result = await invoke<{ id: string }>('startServer', router)
    router.id = result?.id || router.id
    router.status = 'connected'
    saveRouters()
    notify(`${router.name} restarted`, 'success')
  } catch (err: any) {
    router.status = 'error'
    router.error = err.message
    saveRouters()
    notify(`Failed to restart: ${err.message}`, 'error')
  }
}

function edit(id: string) {
  const router = routers.value.find(r => r.id === id)
  editingRouter.value = router || null
}

function cancelEdit() {
  editingRouter.value = null
}

function addRemote(device: { id: string; name?: string; address?: string; host?: string; port?: number }) {
  const { notify } = useNotifications()
  const address = device.address || device.host || ''
  const port = device.port || 7330
  const fullAddress = address.includes(':') ? address : `${address}:${port}`
  const wsAddress = fullAddress.startsWith('ws://') ? fullAddress : `ws://${fullAddress}`

  const existing = routers.value.find(r =>
    r.address === fullAddress || r.address === wsAddress || r.remoteAddress === fullAddress
  )
  if (existing) {
    notify(`Router "${existing.name}" already added`, 'warning')
    return
  }

  const remote: Router = {
    id: `remote-${Date.now()}`,
    type: 'clasp',
    protocol: 'clasp',
    name: device.name || `Remote Router @ ${fullAddress}`,
    address: wsAddress,
    remoteAddress: fullAddress,
    isRemote: true,
    status: 'available',
    discoveredFrom: device.id,
  }

  routers.value.push(remote)
  saveRouters()
  notify(`Added remote router: ${remote.name}`, 'success')
}

const availableRouters = computed(() =>
  routers.value.filter(r =>
    r.isRemote || r.status === 'connected' || r.status === 'running' || r.status === 'available'
  )
)

function handleStatusUpdate(status: { id: string; status: string; error?: string }) {
  const router = routers.value.find(r => r.id === status.id)
  if (!router) return false
  router.status = status.status as any
  if (status.error) router.error = status.error
  return true
}

export function useRouters() {
  return {
    routers: readonly(routers),
    editingRouter: readonly(editingRouter),
    availableRouters,
    restore,
    add,
    remove,
    restart,
    edit,
    cancelEdit,
    addRemote,
    saveRouters,
    handleStatusUpdate,
  }
}
