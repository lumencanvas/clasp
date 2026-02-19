import { ref, computed, readonly } from 'vue'
import { useClasp } from './useClasp.js'
import { useIdentity } from './useIdentity.js'
import { ADDR } from '../lib/constants.js'
import { hashPassword, generateSalt } from '../lib/crypto.js'

// Shared state
const namespaceTree = ref(new Map()) // ns path -> { meta, rooms: Map<roomId, roomInfo>, children: Set<childNs> }
const subscribedNamespaces = ref(new Set(JSON.parse(localStorage.getItem('clasp-chat-pinned-ns') || '[]')))
const unlockedNamespaces = ref(new Set()) // session-scoped, not persisted
const discoveredNamespaces = ref(new Map()) // ns path -> meta (top-level public)

// Active subscriptions
const activeSubscriptions = new Map() // ns path -> unsub function
let unsubTopLevel = null

function persistPinned() {
  localStorage.setItem('clasp-chat-pinned-ns', JSON.stringify([...subscribedNamespaces.value]))
}

/**
 * Sanitize a namespace path to prevent address traversal attacks.
 * - Strips leading/trailing slashes
 * - Rejects '..' segments (directory traversal)
 * - Rejects '*' and '**' (wildcard injection)
 * - Collapses consecutive slashes
 * - Only allows alphanumeric, hyphens, underscores, and single slashes
 * Returns null if the path is invalid.
 */
function sanitizeNsPath(path) {
  if (!path || typeof path !== 'string') return null
  // Trim and strip leading/trailing slashes
  let clean = path.trim().replace(/^\/+|\/+$/g, '')
  // Collapse consecutive slashes
  clean = clean.replace(/\/+/g, '/')
  // Reject empty
  if (!clean) return null
  // Reject traversal, wildcards, and control characters
  const segments = clean.split('/')
  for (const seg of segments) {
    if (seg === '.' || seg === '..') return null
    if (seg === '*' || seg === '**') return null
    if (seg.length === 0) return null
    // Only allow safe characters: alphanumeric, hyphens, underscores
    if (!/^[a-zA-Z0-9_-]+$/.test(seg)) return null
  }
  return clean
}

function ensureNode(ns) {
  if (!namespaceTree.value.has(ns)) {
    namespaceTree.value.set(ns, { meta: null, rooms: new Map(), children: new Set() })
    namespaceTree.value = new Map(namespaceTree.value)
  }
  return namespaceTree.value.get(ns)
}

function parseNamespaceFromAddress(address, basePrefix) {
  // Given address like /chat/registry/ns/gaming/minecraft/room-uuid
  // and basePrefix like /chat/registry/ns/gaming
  // We need to extract the namespace segments and the roomId (last segment)
  const relative = address.slice(basePrefix.length + 1) // "minecraft/room-uuid" or "room-uuid"
  const segments = relative.split('/')
  const roomId = segments.pop() // last segment is always the roomId
  const subNs = segments.length > 0 ? segments.join('/') : null
  return { roomId, subNs }
}

function subscribeNamespace(ns) {
  const { subscribe, connected, client } = useClasp()
  if (!connected.value) return () => {}

  const key = `shallow:${ns}`
  if (activeSubscriptions.has(key)) return activeSubscriptions.get(key)

  ensureNode(ns)

  const pattern = `${ADDR.NS_REGISTRY}/${ns}/*`
  const prefix = `${ADDR.NS_REGISTRY}/${ns}/`

  const unsub = subscribe(pattern, (data, address) => {
    const roomId = address.split('/').pop()
    const node = ensureNode(ns)

    if (data === null) {
      node.rooms.delete(roomId)
    } else {
      node.rooms.set(roomId, { id: roomId, ...data })
    }
    namespaceTree.value = new Map(namespaceTree.value)
  })

  // Process snapshot cache
  if (client.value?.params) {
    for (const [address, data] of client.value.params) {
      if (address.startsWith(prefix) && data !== null) {
        const rest = address.slice(prefix.length)
        // Only direct children (no slashes in rest)
        if (!rest.includes('/')) {
          const node = ensureNode(ns)
          node.rooms.set(rest, { id: rest, ...data })
        }
      }
    }
    namespaceTree.value = new Map(namespaceTree.value)
  }

  activeSubscriptions.set(key, unsub)
  return unsub
}

function subscribeNamespaceDeep(ns) {
  const { subscribe, connected, client } = useClasp()
  if (!connected.value) return () => {}

  const key = `deep:${ns}`
  if (activeSubscriptions.has(key)) return activeSubscriptions.get(key)

  ensureNode(ns)

  const pattern = `${ADDR.NS_REGISTRY}/${ns}/**`
  const prefix = `${ADDR.NS_REGISTRY}/${ns}`

  const unsub = subscribe(pattern, (data, address) => {
    const { roomId, subNs } = parseNamespaceFromAddress(address, prefix)

    if (subNs) {
      // Room in a child namespace
      const fullChildNs = `${ns}/${subNs}`
      const childNode = ensureNode(fullChildNs)

      if (data === null) {
        childNode.rooms.delete(roomId)
      } else {
        childNode.rooms.set(roomId, { id: roomId, ...data, namespace: fullChildNs })
      }

      // Register child in parent
      const parentNode = ensureNode(ns)
      // For nested sub-namespaces, register immediate child only
      const immediateSeg = subNs.split('/')[0]
      parentNode.children.add(`${ns}/${immediateSeg}`)

      // Register intermediate children too
      const parts = subNs.split('/')
      for (let i = 0; i < parts.length - 1; i++) {
        const parentPath = `${ns}/${parts.slice(0, i + 1).join('/')}`
        const childPath = `${ns}/${parts.slice(0, i + 2).join('/')}`
        ensureNode(parentPath).children.add(childPath)
      }
    } else {
      // Room directly in this namespace
      const node = ensureNode(ns)
      if (data === null) {
        node.rooms.delete(roomId)
      } else {
        node.rooms.set(roomId, { id: roomId, ...data, namespace: ns })
      }
    }

    namespaceTree.value = new Map(namespaceTree.value)
  })

  // Also subscribe to namespace metadata for this namespace and children
  const metaPattern = `${ADDR.NS_META}/${ns}/**`
  const metaUnsub = subscribe(metaPattern, (data, address) => {
    const metaNs = address.slice(`${ADDR.NS_META}/`.length)
    if (data !== null) {
      const node = ensureNode(metaNs)
      node.meta = data
      namespaceTree.value = new Map(namespaceTree.value)
    }
  })

  // Subscribe to the direct meta too
  const directMetaUnsub = subscribe(`${ADDR.NS_META}/${ns}`, (data) => {
    if (data !== null) {
      const node = ensureNode(ns)
      node.meta = data
      namespaceTree.value = new Map(namespaceTree.value)
    }
  })

  // Process snapshot cache
  if (client.value?.params) {
    const roomPrefix = `${ADDR.NS_REGISTRY}/${ns}/`
    for (const [address, data] of client.value.params) {
      if (address.startsWith(roomPrefix) && data !== null) {
        const rest = address.slice(roomPrefix.length)
        const segments = rest.split('/')
        const roomId = segments.pop()
        const subNs = segments.length > 0 ? segments.join('/') : null

        if (subNs) {
          const fullChildNs = `${ns}/${subNs}`
          const childNode = ensureNode(fullChildNs)
          childNode.rooms.set(roomId, { id: roomId, ...data, namespace: fullChildNs })

          const parentNode = ensureNode(ns)
          parentNode.children.add(`${ns}/${subNs.split('/')[0]}`)
        } else {
          const node = ensureNode(ns)
          node.rooms.set(roomId, { id: roomId, ...data, namespace: ns })
        }
      }
    }
    namespaceTree.value = new Map(namespaceTree.value)
  }

  const cleanup = () => {
    unsub()
    metaUnsub()
    directMetaUnsub()
  }

  activeSubscriptions.set(key, cleanup)
  return cleanup
}

function discoverTopLevelNamespaces() {
  const { subscribe, connected, client } = useClasp()
  if (!connected.value) return () => {}

  if (unsubTopLevel) unsubTopLevel()

  unsubTopLevel = subscribe(`${ADDR.NS_META}/*`, (data, address) => {
    const ns = address.split('/').pop()
    if (data === null) {
      discoveredNamespaces.value.delete(ns)
    } else if (data.isPublic) {
      discoveredNamespaces.value.set(ns, { path: ns, ...data })
    }
    discoveredNamespaces.value = new Map(discoveredNamespaces.value)
  })

  // Process snapshot cache
  if (client.value?.params) {
    const prefix = `${ADDR.NS_META}/`
    for (const [address, data] of client.value.params) {
      if (address.startsWith(prefix) && data !== null && data.isPublic) {
        const rest = address.slice(prefix.length)
        if (!rest.includes('/')) {
          discoveredNamespaces.value.set(rest, { path: rest, ...data })
        }
      }
    }
    discoveredNamespaces.value = new Map(discoveredNamespaces.value)
  }

  return unsubTopLevel
}

function discoverChildNamespaces(ns) {
  const { subscribe, connected } = useClasp()
  if (!connected.value) return () => {}

  return subscribe(`${ADDR.NS_META}/${ns}/*`, (data, address) => {
    const childNs = address.slice(`${ADDR.NS_META}/`.length)
    if (data === null) {
      discoveredNamespaces.value.delete(childNs)
    } else if (data.isPublic) {
      discoveredNamespaces.value.set(childNs, { path: childNs, ...data })
    }
    discoveredNamespaces.value = new Map(discoveredNamespaces.value)
  })
}

async function createNamespace(path, { description = '', isPublic = true, password = null, icon = '' } = {}) {
  const safePath = sanitizeNsPath(path)
  if (!safePath) return

  const { set, connected } = useClasp()
  const { userId, displayName } = useIdentity()
  if (!connected.value) return

  const meta = {
    description,
    isPublic,
    icon,
    createdBy: userId.value,
    creatorName: displayName.value,
    createdAt: Date.now(),
  }

  if (password) {
    const salt = generateSalt()
    const hash = await hashPassword(password, salt)
    // Store hash/salt at a separate non-browsable path so wildcard subscriptions
    // on ns-meta/* don't leak them to every client.
    set(`${ADDR.NS_META}/${safePath}`, meta)
    set(`${ADDR.NS_META}/${safePath}/__auth`, { passwordHash: hash, passwordSalt: salt })
  } else {
    set(`${ADDR.NS_META}/${safePath}`, meta)
  }

  // Bootstrap tree node
  ensureNode(safePath)
  const node = namespaceTree.value.get(safePath)
  node.meta = meta
  namespaceTree.value = new Map(namespaceTree.value)
}

function registerRoomInNamespace(ns, roomId, roomInfo) {
  const safePath = sanitizeNsPath(ns)
  if (!safePath) return

  const { set, connected } = useClasp()
  if (!connected.value) return

  set(`${ADDR.NS_REGISTRY}/${safePath}/${roomId}`, roomInfo)

  // Update local tree
  const node = ensureNode(safePath)
  node.rooms.set(roomId, { id: roomId, ...roomInfo, namespace: safePath })
  namespaceTree.value = new Map(namespaceTree.value)
}

function removeRoomFromNamespace(ns, roomId) {
  const safePath = sanitizeNsPath(ns)
  if (!safePath) return

  const { set, connected } = useClasp()
  if (!connected.value) return

  set(`${ADDR.NS_REGISTRY}/${safePath}/${roomId}`, null)

  // Update local tree
  if (namespaceTree.value.has(safePath)) {
    namespaceTree.value.get(safePath).rooms.delete(roomId)
    namespaceTree.value = new Map(namespaceTree.value)
  }
}

function pinNamespace(ns) {
  const safePath = sanitizeNsPath(ns)
  if (!safePath) return

  subscribedNamespaces.value.add(safePath)
  subscribedNamespaces.value = new Set(subscribedNamespaces.value)
  persistPinned()
  subscribeNamespaceDeep(safePath)
}

function unpinNamespace(ns) {
  const safePath = sanitizeNsPath(ns)
  if (!safePath) return

  subscribedNamespaces.value.delete(safePath)
  subscribedNamespaces.value = new Set(subscribedNamespaces.value)
  persistPinned()

  // Cleanup subscriptions
  const key = `deep:${safePath}`
  if (activeSubscriptions.has(key)) {
    activeSubscriptions.get(key)()
    activeSubscriptions.delete(key)
  }
}

async function unlockNamespace(ns, password) {
  const safePath = sanitizeNsPath(ns)
  if (!safePath) return false

  const { subscribe, connected } = useClasp()
  if (!connected.value) return false

  // Fetch auth data from the separate __auth path (not exposed via wildcard browse)
  const auth = await new Promise((resolve) => {
    const unsub = subscribe(`${ADDR.NS_META}/${safePath}/__auth`, (data) => {
      resolve(data)
      unsub()
    })
    setTimeout(() => { unsub(); resolve(null) }, 3000)
  })

  if (auth && auth.passwordHash && auth.passwordSalt) {
    const hash = await hashPassword(password, auth.passwordSalt)
    if (hash !== auth.passwordHash) return false
  }

  // Success
  unlockedNamespaces.value.add(safePath)
  unlockedNamespaces.value = new Set(unlockedNamespaces.value)
  pinNamespace(safePath)
  return true
}

function isNamespaceUnlocked(ns) {
  return unlockedNamespaces.value.has(ns)
}

async function lookupNamespace(ns) {
  const safePath = sanitizeNsPath(ns)
  if (!safePath) return null

  const { subscribe, connected } = useClasp()
  if (!connected.value) return null

  // Fetch meta (public info)
  const meta = await new Promise((resolve) => {
    const unsub = subscribe(`${ADDR.NS_META}/${safePath}`, (data) => {
      resolve(data)
      unsub()
    })
    setTimeout(() => { unsub(); resolve(null) }, 3000)
  })

  if (!meta) return null

  // Check if there's an __auth record (indicates password-protected)
  const auth = await new Promise((resolve) => {
    const unsub = subscribe(`${ADDR.NS_META}/${safePath}/__auth`, (data) => {
      resolve(data)
      unsub()
    })
    setTimeout(() => { unsub(); resolve(null) }, 2000)
  })

  // Return meta with a flag indicating password protection, but NOT the hash itself
  if (auth && auth.passwordHash) {
    return { ...meta, hasPassword: true }
  }
  return meta
}

async function deleteNamespace(ns) {
  const safePath = sanitizeNsPath(ns)
  if (!safePath) return

  const { set, connected } = useClasp()
  const { userId } = useIdentity()
  if (!connected.value) return

  const node = namespaceTree.value.get(safePath)
  const meta = node?.meta
  if (meta && meta.createdBy !== userId.value) return

  // Recursively collect all descendant namespace paths
  function collectDescendants(path) {
    const paths = []
    const n = namespaceTree.value.get(path)
    if (n?.children) {
      for (const childPath of n.children) {
        paths.push(childPath)
        paths.push(...collectDescendants(childPath))
      }
    }
    return paths
  }

  const allDescendants = collectDescendants(safePath)

  // Delete all room registry entries under this namespace
  if (node?.rooms) {
    for (const roomId of node.rooms.keys()) {
      set(`${ADDR.NS_REGISTRY}/${safePath}/${roomId}`, null)
    }
  }

  // Delete all descendant namespaces (rooms, meta, auth)
  for (const childPath of allDescendants) {
    const childNode = namespaceTree.value.get(childPath)
    if (childNode?.rooms) {
      for (const roomId of childNode.rooms.keys()) {
        set(`${ADDR.NS_REGISTRY}/${childPath}/${roomId}`, null)
      }
    }
    set(`${ADDR.NS_META}/${childPath}`, null)
    set(`${ADDR.NS_META}/${childPath}/__auth`, null)
    namespaceTree.value.delete(childPath)
    discoveredNamespaces.value.delete(childPath)
  }

  // Delete the namespace metadata and auth
  set(`${ADDR.NS_META}/${safePath}`, null)
  set(`${ADDR.NS_META}/${safePath}/__auth`, null)

  // Remove from local state
  namespaceTree.value.delete(safePath)
  namespaceTree.value = new Map(namespaceTree.value)
  discoveredNamespaces.value.delete(safePath)
  discoveredNamespaces.value = new Map(discoveredNamespaces.value)

  // Unpin if pinned
  unpinNamespace(safePath)
}

async function changeNamespacePassword(ns, newPassword) {
  const safePath = sanitizeNsPath(ns)
  if (!safePath) return

  const { set, connected } = useClasp()
  const { userId } = useIdentity()
  if (!connected.value) return

  const node = namespaceTree.value.get(safePath)
  if (node?.meta?.createdBy && node.meta.createdBy !== userId.value) return

  if (newPassword) {
    const salt = generateSalt()
    const hash = await hashPassword(newPassword, salt)
    set(`${ADDR.NS_META}/${safePath}/__auth`, { passwordHash: hash, passwordSalt: salt })
  } else {
    // Remove password
    set(`${ADDR.NS_META}/${safePath}/__auth`, null)
  }
}

function getChildNamespaces(parentPath) {
  const results = []
  const prefix = parentPath ? `${parentPath}/` : ''
  for (const [path, node] of namespaceTree.value) {
    if (parentPath && path.startsWith(prefix)) {
      // Direct child only (no further slashes after prefix)
      const rest = path.slice(prefix.length)
      if (!rest.includes('/')) {
        results.push({ path, meta: node.meta, roomCount: node.rooms.size })
      }
    }
  }
  // Also check discoveredNamespaces for metadata
  for (const [path, meta] of discoveredNamespaces.value) {
    if (parentPath && path.startsWith(prefix)) {
      const rest = path.slice(prefix.length)
      if (!rest.includes('/') && !results.find(r => r.path === path)) {
        results.push({ path, meta, roomCount: 0 })
      }
    }
  }
  return results.sort((a, b) => a.path.localeCompare(b.path))
}

function searchNamespaces(query) {
  if (!query || !query.trim()) return { namespaces: [], rooms: [] }
  const q = query.trim().toLowerCase()
  const namespaces = []
  const rooms = []

  // Search discovered namespaces
  for (const [path, meta] of discoveredNamespaces.value) {
    if (
      path.toLowerCase().includes(q) ||
      (meta.description || '').toLowerCase().includes(q)
    ) {
      namespaces.push({ path, ...meta })
    }
  }

  // Search tree nodes (includes pinned private namespaces)
  for (const [path, node] of namespaceTree.value) {
    if (!namespaces.find(n => n.path === path)) {
      if (
        path.toLowerCase().includes(q) ||
        (node.meta?.description || '').toLowerCase().includes(q)
      ) {
        namespaces.push({ path, ...(node.meta || {}), roomCount: node.rooms.size })
      }
    }
    // Search rooms within this namespace
    for (const [roomId, room] of node.rooms) {
      if ((room.name || '').toLowerCase().includes(q)) {
        rooms.push({ ...room, id: roomId, namespace: path })
      }
    }
  }

  return { namespaces, rooms }
}

function updateNamespaceMeta(ns, updates) {
  const safePath = sanitizeNsPath(ns)
  if (!safePath) return

  const { set, connected } = useClasp()
  const { userId } = useIdentity()
  if (!connected.value) return

  const node = namespaceTree.value.get(safePath)
  const currentMeta = node?.meta || {}

  // Client-side creator check: only the namespace creator can update metadata
  if (currentMeta.createdBy && currentMeta.createdBy !== userId.value) return

  const newMeta = { ...currentMeta, ...updates }

  set(`${ADDR.NS_META}/${safePath}`, newMeta)

  if (node) {
    node.meta = newMeta
    namespaceTree.value = new Map(namespaceTree.value)
  }
}

function initPinnedNamespaces() {
  for (const ns of subscribedNamespaces.value) {
    subscribeNamespaceDeep(ns)
  }
}

function getRoomNamespace(roomId) {
  for (const [ns, node] of namespaceTree.value) {
    if (node.rooms.has(roomId)) return ns
  }
  return null
}

const namespacedRoomIds = computed(() => {
  const ids = new Set()
  for (const [, node] of namespaceTree.value) {
    for (const roomId of node.rooms.keys()) {
      ids.add(roomId)
    }
  }
  return ids
})

const pinnedNamespaceList = computed(() => {
  return [...subscribedNamespaces.value].sort()
})

export function useNamespaces() {
  return {
    sanitizeNsPath,
    namespaceTree: readonly(namespaceTree),
    subscribedNamespaces: readonly(subscribedNamespaces),
    unlockedNamespaces: readonly(unlockedNamespaces),
    discoveredNamespaces: readonly(discoveredNamespaces),
    namespacedRoomIds,
    pinnedNamespaceList,
    subscribeNamespace,
    subscribeNamespaceDeep,
    discoverTopLevelNamespaces,
    discoverChildNamespaces,
    createNamespace,
    registerRoomInNamespace,
    removeRoomFromNamespace,
    pinNamespace,
    unpinNamespace,
    unlockNamespace,
    isNamespaceUnlocked,
    lookupNamespace,
    updateNamespaceMeta,
    deleteNamespace,
    changeNamespacePassword,
    getChildNamespaces,
    searchNamespaces,
    initPinnedNamespaces,
    getRoomNamespace,
  }
}
