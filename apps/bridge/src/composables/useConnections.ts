import { ref, readonly } from 'vue'
import type { Connection, Protocol } from '../lib/types'
import { useElectron } from './useElectron'
import { loadFromStorage, saveToStorage } from './useStorage'
import { useNotifications } from './useNotifications'
import { useRouters } from './useRouters'

const connections = ref<Connection[]>([])
const editingConnection = ref<Connection | null>(null)

function loadConnections(): Connection[] {
  const saved = loadFromStorage<Partial<Connection>[]>('servers', [])
  return saved.map(s => ({ ...s, status: 'disconnected' } as Connection))
}

function saveConnections() {
  const toSave = connections.value.map(s => ({
    id: s.id, type: s.type, protocol: s.protocol, name: s.name,
    address: s.address, routerId: s.routerId,
    bind: s.bind, port: s.port, host: s.host, topics: s.topics,
    mode: s.mode, basePath: s.basePath, cors: s.cors,
    subnet: s.subnet, universe: s.universe, serialPort: s.serialPort,
    token: s.token, namespace: s.namespace,
    inputPort: s.inputPort, outputPort: s.outputPort,
    clientId: s.clientId, qos: s.qos, keepAlive: s.keepAlive,
    authEnabled: s.authEnabled, username: s.username, password: s.password,
    format: s.format, pingInterval: s.pingInterval,
    universes: s.universes, sourceName: s.sourceName, priority: s.priority,
    multicast: s.multicast, bindAddress: s.bindAddress,
    unicastDestinations: s.unicastDestinations,
  }))
  saveToStorage('servers', toSave)
}

async function restore() {
  const saved = loadConnections()
  const { invoke } = useElectron()

  for (const config of saved) {
    try {
      const result = await invoke<{ id: string }>('startServer', config)
      config.id = result?.id || config.id
      config.status = 'connected'
    } catch (err: any) {
      config.status = 'error'
      config.error = err.message
    }
    connections.value.push(config)
  }
}

async function add(config: Partial<Connection>) {
  const { notify } = useNotifications()
  const { invoke } = useElectron()
  const { availableRouters } = useRouters()
  const isEditing = editingConnection.value !== null

  // Find target router
  let targetRouter = config.routerId
    ? availableRouters.value.find(r => r.id === config.routerId)
    : availableRouters.value[0]

  if (!targetRouter) {
    notify('No CLASP router available. Please add a router first.', 'error')
    return false
  }

  const connection: Connection = {
    id: isEditing ? editingConnection.value!.id : Date.now().toString(),
    status: 'starting',
    routerId: targetRouter.id,
    ...config,
  } as Connection

  try {
    const result = await invoke<{ id: string }>('startServer', connection)
    connection.id = result?.id || connection.id
    connection.status = 'connected'

    if (isEditing) {
      const idx = connections.value.findIndex(s => s.id === connection.id)
      if (idx !== -1) connections.value[idx] = connection
      else connections.value.push(connection)
    } else {
      connections.value.push(connection)
    }
    editingConnection.value = null
    saveConnections()
    return true
  } catch (err: any) {
    connection.status = 'error'
    connection.error = err.message

    if (isEditing) {
      const idx = connections.value.findIndex(s => s.id === connection.id)
      if (idx !== -1) connections.value[idx] = connection
      else connections.value.push(connection)
    } else {
      connections.value.push(connection)
    }
    editingConnection.value = null
    saveConnections()
    notify(`Failed to start connection: ${err.message}`, 'error')
    return false
  }
}

async function remove(id: string) {
  const { invoke } = useElectron()
  try {
    await invoke('stopServer', id)
  } catch (err) {
    console.error('Failed to stop connection:', err)
  }
  connections.value = connections.value.filter(s => s.id !== id)
  saveConnections()
}

async function restart(id: string) {
  const { invoke } = useElectron()
  const { notify } = useNotifications()
  const conn = connections.value.find(s => s.id === id)
  if (!conn) return

  conn.status = 'reconnecting'
  conn.error = undefined
  notify(`Restarting ${conn.name}...`, 'info')

  try {
    await invoke('stopServer', id)
    await new Promise(resolve => setTimeout(resolve, 500))
    const result = await invoke<{ id: string }>('startServer', conn)
    conn.id = result?.id || conn.id
    conn.status = 'connected'
    conn.error = undefined
    saveConnections()
    notify(`${conn.name} restarted successfully`, 'success')
  } catch (err: any) {
    conn.status = 'error'
    conn.error = err.message
    saveConnections()
    notify(`Failed to restart ${conn.name}: ${err.message}`, 'error')
  }
}

function edit(id: string) {
  editingConnection.value = connections.value.find(s => s.id === id) || null
}

function cancelEdit() {
  editingConnection.value = null
}

function handleStatusUpdate(status: { id: string; status: string; error?: string }): boolean {
  const conn = connections.value.find(s => s.id === status.id)
  if (!conn) return false
  conn.status = status.status as any
  if (status.error) conn.error = status.error
  return true
}

function handleRouterStatus(status: { bridgeId: string; connected: boolean; error?: string; routerId?: string }) {
  const conn = connections.value.find(s => s.id === status.bridgeId)
  if (conn) {
    conn.routerConnected = status.connected
    conn.routerError = status.error || undefined
    conn.connectedRouterId = status.routerId || undefined
  }
}

export function useConnections() {
  return {
    connections: readonly(connections),
    editingConnection: readonly(editingConnection),
    restore,
    add,
    remove,
    restart,
    edit,
    cancelEdit,
    saveConnections,
    handleStatusUpdate,
    handleRouterStatus,
  }
}
