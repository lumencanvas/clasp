<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { ROOM_TYPE_INFO } from '../lib/constants.js'
import { formatRelativeTime } from '../lib/utils.js'
import { useNamespaces } from '../composables/useNamespaces.js'
import { useIdentity } from '../composables/useIdentity.js'
import NamespaceCard from './NamespaceCard.vue'
import NamespaceCreateDialog from './NamespaceCreateDialog.vue'

const props = defineProps({
  rooms: { type: Array, default: () => [] },
  joinedRoomIds: { type: Set, default: () => new Set() },
})

const emit = defineEmits(['join', 'close'])

const { userId } = useIdentity()

const {
  discoveredNamespaces,
  subscribedNamespaces,
  unlockedNamespaces,
  namespaceTree,
  discoverTopLevelNamespaces,
  discoverChildNamespaces,
  subscribeNamespace,
  pinNamespace,
  unpinNamespace,
  lookupNamespace,
  unlockNamespace,
  namespacedRoomIds,
  updateNamespaceMeta,
  createNamespace,
  deleteNamespace,
  changeNamespacePassword,
  searchNamespaces,
} = useNamespaces()

// Navigation state
const currentView = ref('top') // 'top' or a namespace path
const viewStack = ref([])

// Private namespace unlock
const nsPathInput = ref('')
const nsLookupError = ref('')
const nsPasswordPrompt = ref(null) // namespace meta when password needed
const nsPasswordInput = ref('')
const nsPasswordError = ref('')

// Room password prompt
const passwordPromptRoom = ref(null)
const passwordInput = ref('')
const passwordError = ref('')

// Namespace settings editing
const showNsSettings = ref(false)
const nsEditDesc = ref('')
const nsEditIcon = ref('')
const nsEditPublic = ref(true)
const nsEditPassword = ref('')
const showNsDeleteConfirm = ref(false)

// Search
const searchQuery = ref('')
let searchDebounce = null

// Create namespace dialog
const showCreateNs = ref(false)

let childUnsub = null

// Discovered namespaces as list
const namespaceList = computed(() => {
  return [...discoveredNamespaces.value.values()]
})

// Current namespace data (for drill-in)
const currentNsNode = computed(() => {
  if (currentView.value === 'top') return null
  return namespaceTree.value.get(currentView.value) || null
})

const currentNsMeta = computed(() => {
  return currentNsNode.value?.meta || discoveredNamespaces.value.get(currentView.value) || null
})

const currentNsRooms = computed(() => {
  if (!currentNsNode.value) return []
  return [...currentNsNode.value.rooms.values()].sort((a, b) => (a.name || '').localeCompare(b.name || ''))
})

const currentNsChildren = computed(() => {
  if (!currentNsNode.value?.children) return []
  const children = []
  for (const childPath of currentNsNode.value.children) {
    const meta = discoveredNamespaces.value.get(childPath)
    if (meta) {
      children.push(meta)
    } else {
      // Construct from tree node
      const node = namespaceTree.value.get(childPath)
      children.push({
        path: childPath,
        description: node?.meta?.description || '',
        creatorName: node?.meta?.creatorName || '',
        icon: node?.meta?.icon || '',
      })
    }
  }
  return children.sort((a, b) => a.path.localeCompare(b.path))
})

// Search results — grouped by top-level namespace
const searchResults = computed(() => {
  if (!searchQuery.value.trim()) return null
  const raw = searchNamespaces(searchQuery.value)

  // Group namespaces and rooms by their top-level namespace
  const groups = new Map() // topLevel -> { namespace meta, matchingChildren: [], matchingRooms: [] }

  for (const ns of raw.namespaces) {
    const topLevel = ns.path.split('/')[0]
    if (!groups.has(topLevel)) {
      groups.set(topLevel, { matchingChildren: [], matchingRooms: [] })
    }
    groups.get(topLevel).matchingChildren.push(ns)
  }

  for (const room of raw.rooms) {
    const topLevel = (room.namespace || '').split('/')[0] || '__uncategorized'
    if (!groups.has(topLevel)) {
      groups.set(topLevel, { matchingChildren: [], matchingRooms: [] })
    }
    groups.get(topLevel).matchingRooms.push(room)
  }

  // Build sorted list of top-level results with metadata
  const results = []
  for (const [topLevel, group] of groups) {
    // Find metadata for the top-level namespace
    const meta = discoveredNamespaces.value.get(topLevel)
      || namespaceTree.value.get(topLevel)?.meta
      || null
    results.push({
      path: topLevel,
      icon: meta?.icon || '',
      description: meta?.description || '',
      creatorName: meta?.creatorName || '',
      isTopLevel: true,
      matchCount: group.matchingChildren.length + group.matchingRooms.length,
      matchingChildren: group.matchingChildren,
      matchingRooms: group.matchingRooms,
    })
  }

  return results.sort((a, b) => a.path.localeCompare(b.path))
})

const isSearching = computed(() => !!searchQuery.value.trim())

// Breadcrumb segments for current view
const breadcrumbs = computed(() => {
  if (currentView.value === 'top') return []
  const parts = currentView.value.split('/')
  const crumbs = [{ label: 'Home', path: 'top' }]
  for (let i = 0; i < parts.length; i++) {
    crumbs.push({
      label: parts[i],
      path: parts.slice(0, i + 1).join('/'),
    })
  }
  return crumbs
})

// Uncategorized rooms = public rooms NOT in any namespace
const uncategorizedRooms = computed(() => {
  return props.rooms.filter(r => !namespacedRoomIds.value.has(r.id))
})

function getNsRoomCount(nsPath) {
  const node = namespaceTree.value.get(nsPath)
  return node?.rooms?.size || 0
}

function navigateToNs(nsPath) {
  viewStack.value.push(currentView.value)
  currentView.value = nsPath

  // Subscribe to child namespace metadata AND rooms in this namespace
  if (childUnsub) childUnsub()
  childUnsub = discoverChildNamespaces(nsPath)
  // Subscribe to rooms — deduped internally by activeSubscriptions, so no need to unsub on navigate
  subscribeNamespace(nsPath)
}

function navigateBack() {
  if (viewStack.value.length > 0) {
    currentView.value = viewStack.value.pop()
  } else {
    currentView.value = 'top'
  }
  if (childUnsub) { childUnsub(); childUnsub = null }
}

function isNsPinned(nsPath) {
  return subscribedNamespaces.value.has(nsPath)
}

async function handlePinNs(nsPath) {
  // Already unlocked this session — pin directly
  if (unlockedNamespaces.value.has(nsPath)) {
    pinNamespace(nsPath)
    return
  }

  // Look up namespace to check for password
  const meta = await lookupNamespace(nsPath)
  if (!meta) {
    // Couldn't fetch meta — pin anyway (public namespace without meta record)
    pinNamespace(nsPath)
    return
  }

  // Creator bypass — no password prompt needed
  if (meta.createdBy === userId.value) {
    pinNamespace(nsPath)
    return
  }

  if (meta.hasPassword) {
    // Show password prompt
    nsPasswordPrompt.value = { path: nsPath, ...meta }
    nsPasswordInput.value = ''
    nsPasswordError.value = ''
  } else {
    pinNamespace(nsPath)
  }
}

function handleUnpinNs(nsPath) {
  unpinNamespace(nsPath)
}

// Private namespace lookup
async function handleLookupNs() {
  const path = nsPathInput.value.trim().replace(/^\/+|\/+$/g, '')
  if (!path) return

  nsLookupError.value = ''
  const meta = await lookupNamespace(path)

  if (!meta) {
    nsLookupError.value = 'Group not found'
    return
  }

  if (meta.hasPassword) {
    // Needs password
    nsPasswordPrompt.value = { path, ...meta }
    nsPasswordInput.value = ''
    nsPasswordError.value = ''
  } else {
    // No password — pin directly
    pinNamespace(path)
    nsPathInput.value = ''
  }
}

async function handleNsPasswordSubmit() {
  if (!nsPasswordPrompt.value || !nsPasswordInput.value) return
  nsPasswordError.value = ''

  const success = await unlockNamespace(nsPasswordPrompt.value.path, nsPasswordInput.value)
  if (success) {
    nsPasswordPrompt.value = null
    nsPasswordInput.value = ''
    nsPathInput.value = ''
  } else {
    nsPasswordError.value = 'Incorrect password'
  }
}

// Room join handling
function handleJoinClick(room) {
  if (room.hasPassword && !props.joinedRoomIds.has(room.id)) {
    passwordPromptRoom.value = room
    passwordInput.value = ''
    passwordError.value = ''
  } else {
    emit('join', room.id)
  }
}

function submitPassword() {
  if (!passwordInput.value) return
  emit('join', passwordPromptRoom.value.id, passwordInput.value)
  passwordPromptRoom.value = null
  passwordInput.value = ''
}

const isNsCreator = computed(() => {
  return currentNsMeta.value?.createdBy === userId.value
})

function openNsSettings() {
  nsEditDesc.value = currentNsMeta.value?.description || ''
  nsEditIcon.value = currentNsMeta.value?.icon || ''
  nsEditPublic.value = currentNsMeta.value?.isPublic !== false
  showNsSettings.value = true
}

function saveNsSettings() {
  if (currentView.value === 'top') return
  updateNamespaceMeta(currentView.value, {
    description: nsEditDesc.value,
    icon: nsEditIcon.value,
    isPublic: nsEditPublic.value,
  })
  showNsSettings.value = false
}

async function handleChangePassword() {
  if (currentView.value === 'top') return
  await changeNamespacePassword(currentView.value, nsEditPassword.value || null)
  nsEditPassword.value = ''
  showNsSettings.value = false
}

async function handleDeleteNamespace() {
  if (currentView.value === 'top') return
  await deleteNamespace(currentView.value)
  showNsDeleteConfirm.value = false
  showNsSettings.value = false
  currentView.value = 'top'
  viewStack.value = []
}

async function handleCreateNamespace(data) {
  await createNamespace(data.path, {
    description: data.description,
    isPublic: data.isPublic,
    password: data.password,
    icon: data.icon,
  })
  showCreateNs.value = false
}

function navigateToBreadcrumb(path) {
  if (path === 'top') {
    currentView.value = 'top'
    viewStack.value = []
  } else {
    // Build stack from breadcrumb path
    const parts = path.split('/')
    viewStack.value = ['top']
    for (let i = 1; i < parts.length; i++) {
      viewStack.value.push(parts.slice(0, i).join('/'))
    }
    currentView.value = path
  }
  if (childUnsub) { childUnsub(); childUnsub = null }
  if (path !== 'top') {
    childUnsub = discoverChildNamespaces(path)
    subscribeNamespace(path)
  }
}

function clearSearch() {
  searchQuery.value = ''
}

onMounted(() => {
  discoverTopLevelNamespaces()
})

onUnmounted(() => {
  if (childUnsub) childUnsub()
})
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('close')" @keydown.escape="emit('close')" tabindex="-1">
    <div class="dialog">
      <div class="dialog-header">
        <div class="header-nav">
          <button v-if="currentView !== 'top'" class="back-btn" @click="navigateBack">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="19" y1="12" x2="5" y2="12"/>
              <polyline points="12 19 5 12 12 5"/>
            </svg>
          </button>
          <h3 v-if="currentView === 'top'">Browse Channels</h3>
          <div v-else class="breadcrumb-nav">
            <template v-for="(crumb, i) in breadcrumbs" :key="crumb.path">
              <span v-if="i > 0" class="breadcrumb-sep">/</span>
              <button
                :class="['breadcrumb-item', { current: i === breadcrumbs.length - 1 }]"
                @click="navigateToBreadcrumb(crumb.path)"
              >{{ crumb.label }}</button>
            </template>
          </div>
          <div class="header-actions">
            <button
              v-if="currentView !== 'top' && isNsCreator"
              class="ns-action-btn"
              title="Namespace settings"
              @click="openNsSettings"
            >Settings</button>
            <button
              v-if="currentView !== 'top'"
              :class="['pin-toggle', { pinned: isNsPinned(currentView) }]"
              @click="isNsPinned(currentView) ? handleUnpinNs(currentView) : handlePinNs(currentView)"
              :title="isNsPinned(currentView) ? 'This group is in your sidebar' : 'Add this group to your sidebar'"
            >{{ isNsPinned(currentView) ? 'Pinned' : 'Pin to Sidebar' }}</button>
          </div>
        </div>
        <button class="close-btn" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- Search bar -->
      <div class="search-bar">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search groups and channels..."
          autocomplete="off"
        />
        <button v-if="searchQuery" class="search-clear" @click="clearSearch">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div class="dialog-body">
        <p v-if="currentView === 'top' && !isSearching" class="browse-explainer">
          Channels are organized into groups. Pin a group to your sidebar to stay connected.
        </p>

        <!-- SEARCH RESULTS (grouped by top-level namespace) -->
        <template v-if="isSearching && searchResults">
          <div v-if="searchResults.length" class="search-results">
            <div v-for="group in searchResults" :key="group.path" class="search-group">
              <button class="search-group-header" @click="clearSearch(); navigateToNs(group.path)">
                <span v-if="group.icon" class="search-group-icon">{{ group.icon }}</span>
                <span v-else class="search-group-icon search-group-icon-default">/</span>
                <span class="search-group-name">{{ group.path }}</span>
                <span class="search-group-count">{{ group.matchCount }} match{{ group.matchCount !== 1 ? 'es' : '' }}</span>
                <svg class="search-group-arrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                  <polyline points="9 18 15 12 9 6"/>
                </svg>
              </button>
              <div v-if="group.description" class="search-group-desc">{{ group.description }}</div>
              <!-- Preview of matching items inside -->
              <div class="search-group-preview">
                <span
                  v-for="child in group.matchingChildren.slice(0, 3)"
                  :key="child.path"
                  class="search-preview-tag ns-tag"
                >{{ child.path }}</span>
                <span
                  v-for="room in group.matchingRooms.slice(0, 3)"
                  :key="room.id"
                  class="search-preview-tag room-tag"
                ># {{ room.name }}</span>
                <span
                  v-if="group.matchCount > 6"
                  class="search-preview-more"
                >+{{ group.matchCount - 6 }} more</span>
              </div>
            </div>
          </div>
          <div v-else class="empty">
            <p>No results for "{{ searchQuery }}"</p>
            <span>Try a different search term</span>
          </div>
        </template>

        <!-- TOP LEVEL VIEW -->
        <template v-else-if="currentView === 'top'">
          <!-- Action row: Create namespace -->
          <div class="action-row">
            <button class="create-ns-btn" @click="showCreateNs = true">+ New Group</button>
          </div>

          <!-- Namespaces section -->
          <div v-if="namespaceList.length" class="browse-section">
            <div class="browse-label">Channel Groups</div>
            <div class="ns-grid">
              <NamespaceCard
                v-for="ns in namespaceList"
                :key="ns.path"
                :namespace="ns"
                :is-pinned="isNsPinned(ns.path)"
                :room-count="getNsRoomCount(ns.path)"
                @click="navigateToNs"
                @pin="handlePinNs"
                @unpin="handleUnpinNs"
              />
            </div>
          </div>

          <!-- Private namespace entry -->
          <div class="browse-section private-ns">
            <div class="browse-label">Join a Private Group</div>
            <p class="private-ns-hint">Know the name of a private group? Enter it here to request access.</p>
            <form class="ns-lookup-row" @submit.prevent="handleLookupNs">
              <input
                v-model="nsPathInput"
                type="text"
                placeholder="group name, e.g. secret-club"
                autocomplete="off"
              />
              <button type="submit" :disabled="!nsPathInput.trim()">Unlock</button>
            </form>
            <p v-if="nsLookupError" class="lookup-error">{{ nsLookupError }}</p>
          </div>

          <!-- Uncategorized rooms -->
          <div v-if="uncategorizedRooms.length" class="browse-section">
            <div class="browse-label">Channels</div>
            <div class="room-grid">
              <div v-for="room in uncategorizedRooms" :key="room.id" class="discovery-card">
                <div class="card-top">
                  <span class="card-type">{{ (ROOM_TYPE_INFO[room.type] || ROOM_TYPE_INFO.text).label }}</span>
                  <span v-if="room.hasPassword" class="card-lock" title="Password protected">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12">
                      <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
                      <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                    </svg>
                  </span>
                  <span class="card-time">{{ formatRelativeTime(room.createdAt) }}</span>
                </div>
                <h4 class="card-name">{{ room.name }}</h4>
                <p class="card-creator">by {{ room.creatorName || 'Unknown' }}</p>
                <button
                  class="join-btn"
                  :disabled="joinedRoomIds.has(room.id)"
                  @click="handleJoinClick(room)"
                >{{ joinedRoomIds.has(room.id) ? 'Joined' : 'Join' }}</button>
              </div>
            </div>
          </div>

          <div v-if="!namespaceList.length && !uncategorizedRooms.length" class="empty">
            <p>No public channels yet</p>
            <span>Create a group to organize channels, or create a standalone channel.</span>
          </div>
        </template>

        <!-- NAMESPACE DRILL-IN VIEW -->
        <template v-else>
          <p v-if="currentNsMeta?.description" class="ns-description">{{ currentNsMeta.description }}</p>

          <!-- Sub-namespaces -->
          <div v-if="currentNsChildren.length" class="browse-section">
            <div class="browse-label">Sub-groups</div>
            <div class="ns-grid">
              <NamespaceCard
                v-for="child in currentNsChildren"
                :key="child.path"
                :namespace="child"
                :is-pinned="isNsPinned(child.path)"
                :room-count="getNsRoomCount(child.path)"
                @click="navigateToNs"
                @pin="handlePinNs"
                @unpin="handleUnpinNs"
              />
            </div>
          </div>

          <!-- Rooms in namespace -->
          <div class="browse-section">
            <div class="browse-label">Channels in {{ currentView.split('/').pop() }}</div>
            <div v-if="!currentNsRooms.length" class="empty-small">
              <span>No channels in this group yet. Create a channel and assign it to this group.</span>
            </div>
            <div v-else class="room-grid">
              <div v-for="room in currentNsRooms" :key="room.id" class="discovery-card">
                <div class="card-top">
                  <span class="card-type">{{ (ROOM_TYPE_INFO[room.type] || ROOM_TYPE_INFO.text).label }}</span>
                  <span v-if="room.hasPassword" class="card-lock">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12">
                      <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
                      <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                    </svg>
                  </span>
                  <span class="card-time">{{ formatRelativeTime(room.createdAt) }}</span>
                </div>
                <h4 class="card-name">{{ room.name }}</h4>
                <p class="card-creator">by {{ room.creatorName || 'Unknown' }}</p>
                <button
                  class="join-btn"
                  :disabled="joinedRoomIds.has(room.id)"
                  @click="handleJoinClick(room)"
                >{{ joinedRoomIds.has(room.id) ? 'Joined' : 'Join' }}</button>
              </div>
            </div>
          </div>
        </template>

        <!-- Room password prompt overlay -->
        <div v-if="passwordPromptRoom" class="password-overlay" @click.self="passwordPromptRoom = null">
          <div class="password-dialog">
            <h4>Enter Room Password</h4>
            <p>{{ passwordPromptRoom.name }} is password-protected</p>
            <form @submit.prevent="submitPassword">
              <input
                v-model="passwordInput"
                type="password"
                placeholder="Room password"
                autocomplete="off"
                autofocus
              />
              <div v-if="passwordError" class="pw-error">{{ passwordError }}</div>
              <div class="pw-actions">
                <button type="button" class="pw-cancel" @click="passwordPromptRoom = null">Cancel</button>
                <button type="submit" class="pw-submit" :disabled="!passwordInput">Join</button>
              </div>
            </form>
          </div>
        </div>

        <!-- Namespace settings overlay -->
        <div v-if="showNsSettings" class="password-overlay" @click.self="showNsSettings = false">
          <div class="password-dialog" style="width: 380px">
            <h4>Group Settings</h4>
            <p>{{ currentView }}</p>
            <div class="ns-settings-form">
              <label class="ns-settings-label">Description</label>
              <input v-model="nsEditDesc" type="text" placeholder="Description" />
              <label class="ns-settings-label">Icon</label>
              <input v-model="nsEditIcon" type="text" placeholder="Icon character" maxlength="2" />
              <label class="ns-settings-label">Visibility</label>
              <div class="ns-segmented-control">
                <button type="button" :class="['ns-seg-btn', { active: nsEditPublic }]" @click="nsEditPublic = true">Public</button>
                <button type="button" :class="['ns-seg-btn', { active: !nsEditPublic }]" @click="nsEditPublic = false">Private</button>
              </div>
              <span class="ns-visibility-hint">{{ nsEditPublic ? 'Anyone can find and browse this group' : 'Only people who know the name can find it' }}</span>
              <div class="pw-actions">
                <button class="pw-cancel" @click="showNsSettings = false">Cancel</button>
                <button class="pw-submit" @click="saveNsSettings">Save</button>
              </div>

              <hr class="ns-settings-divider" />

              <label class="ns-settings-label">Change Password</label>
              <div class="ns-pw-row">
                <input v-model="nsEditPassword" type="password" placeholder="New password (empty to remove)" autocomplete="off" />
                <button class="pw-change-btn" @click="handleChangePassword">Update</button>
              </div>

              <hr class="ns-settings-divider" />

              <div class="ns-danger-zone">
                <label class="ns-settings-label ns-danger-label">Danger Zone</label>
                <template v-if="!showNsDeleteConfirm">
                  <button class="ns-delete-btn" @click="showNsDeleteConfirm = true">Delete Group</button>
                </template>
                <template v-else>
                  <p class="ns-delete-warning">This will delete this group, all channel registry entries, and sub-groups. This cannot be undone.</p>
                  <div class="pw-actions">
                    <button class="pw-cancel" @click="showNsDeleteConfirm = false">Cancel</button>
                    <button class="ns-delete-confirm" @click="handleDeleteNamespace">Delete Forever</button>
                  </div>
                </template>
              </div>
            </div>
          </div>
        </div>

        <!-- Create namespace dialog -->
        <NamespaceCreateDialog
          v-if="showCreateNs"
          @create="handleCreateNamespace"
          @close="showCreateNs = false"
        />

        <!-- Namespace password prompt overlay -->
        <div v-if="nsPasswordPrompt" class="password-overlay" @click.self="nsPasswordPrompt = null">
          <div class="password-dialog">
            <h4>Unlock Group</h4>
            <p>"{{ nsPasswordPrompt.path }}" is password-protected</p>
            <form @submit.prevent="handleNsPasswordSubmit">
              <input
                v-model="nsPasswordInput"
                type="password"
                placeholder="Group password"
                autocomplete="off"
                autofocus
              />
              <div v-if="nsPasswordError" class="pw-error">{{ nsPasswordError }}</div>
              <div class="pw-actions">
                <button type="button" class="pw-cancel" @click="nsPasswordPrompt = null">Cancel</button>
                <button type="submit" class="pw-submit" :disabled="!nsPasswordInput">Unlock</button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  padding: 1rem;
}

.dialog {
  width: 100%;
  max-width: 620px;
  max-height: 80dvh;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.header-nav {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1;
  min-width: 0;
}

.header-nav h3 {
  font-size: 1rem;
  letter-spacing: 0.06em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.back-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: none;
  color: var(--text-muted);
  border-radius: 4px;
  flex-shrink: 0;
}

.back-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.back-btn svg {
  width: 16px;
  height: 16px;
}

.pin-toggle {
  padding: 0.3rem 0.6rem;
  font-size: 0.7rem;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: 4px;
  flex-shrink: 0;
}

.pin-toggle.pinned {
  background: var(--bg-active);
  color: var(--text-muted);
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: none;
  border: none;
  color: var(--text-muted);
  border-radius: 4px;
  flex-shrink: 0;
}

.close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.close-btn svg {
  width: 16px;
  height: 16px;
}

.dialog-body {
  padding: 1.25rem;
  overflow-y: auto;
}

.browse-section {
  margin-bottom: 1.25rem;
}

.browse-label {
  font-size: 0.75rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted);
  font-weight: 700;
  margin-bottom: 0.5rem;
}

.ns-description {
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin-bottom: 1rem;
}

.ns-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 0.75rem;
}

@media (max-width: 480px) {
  .ns-grid {
    grid-template-columns: 1fr;
  }
}

.private-ns {
  background: var(--bg-tertiary);
  border-radius: 6px;
  padding: 1rem;
}

.ns-lookup-row {
  display: flex;
  gap: 0.5rem;
}

.ns-lookup-row input {
  flex: 1;
  padding: 0.6rem 0.8rem;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.8rem;
  min-width: 0;
}

.ns-lookup-row input:focus {
  outline: none;
  border-color: var(--accent);
}

.ns-lookup-row button {
  padding: 0.6rem 1rem;
  background: var(--accent2);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.8rem;
  white-space: nowrap;
}

.ns-lookup-row button:disabled {
  opacity: 0.5;
}

.lookup-error {
  font-size: 0.75rem;
  color: var(--danger);
  margin-top: 0.4rem;
}

.room-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 0.75rem;
}

@media (max-width: 480px) {
  .room-grid {
    grid-template-columns: 1fr;
  }
}

.discovery-card {
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-type {
  font-size: 0.75rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--accent2);
}

.card-time {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.card-name {
  font-size: 0.95rem;
  letter-spacing: 0.04em;
}

.card-creator {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.card-lock {
  display: flex;
  align-items: center;
  color: var(--accent4);
}

.join-btn {
  margin-top: 0.5rem;
  padding: 0.5rem;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.8rem;
  transition: opacity 0.15s;
}

.join-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.join-btn:disabled {
  background: var(--bg-active);
  color: var(--text-muted);
  cursor: default;
}

.empty {
  text-align: center;
  padding: 2rem;
}

.empty p {
  font-size: 0.85rem;
  color: var(--text-secondary);
  margin-bottom: 0.25rem;
}

.empty span {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.empty-small {
  text-align: center;
  padding: 1rem;
}

.empty-small span {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.password-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: calc(var(--z-modal) + 1);
}

.password-dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1.5rem;
  width: 320px;
  max-width: 90vw;
}

.password-dialog h4 {
  font-size: 0.9rem;
  margin-bottom: 0.25rem;
}

.password-dialog p {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-bottom: 1rem;
}

.password-dialog input {
  width: 100%;
  padding: 0.6rem 0.8rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.85rem;
  margin-bottom: 0.5rem;
  box-sizing: border-box;
}

.password-dialog input:focus {
  outline: none;
  border-color: var(--accent);
}

.pw-error {
  font-size: 0.75rem;
  color: var(--danger);
  margin-bottom: 0.5rem;
}

.pw-actions {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
}

.pw-cancel,
.pw-submit {
  padding: 0.5rem 1rem;
  border-radius: 4px;
  font-size: 0.8rem;
  border: none;
}

.pw-cancel {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.pw-submit {
  background: var(--accent);
  color: white;
}

.pw-submit:disabled {
  opacity: 0.5;
}

/* Namespace settings */
.ns-settings-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: none;
  color: var(--text-muted);
  border-radius: 4px;
  flex-shrink: 0;
}

.ns-settings-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.ns-settings-form {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.ns-settings-label {
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-muted);
  font-weight: 700;
}

.ns-settings-form input {
  width: 100%;
  padding: 0.5rem 0.7rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.8rem;
  box-sizing: border-box;
}

.ns-settings-form input:focus {
  outline: none;
  border-color: var(--accent);
}

.ns-segmented-control {
  display: flex;
  background: var(--bg-active);
  border-radius: 6px;
  padding: 2px;
  gap: 2px;
}

.ns-seg-btn {
  flex: 1;
  padding: 0.35rem 0.75rem;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-muted);
  font-family: var(--font-body);
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.ns-seg-btn.active {
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-weight: 600;
}

.ns-seg-btn:hover:not(.active) {
  color: var(--text-secondary);
}

.ns-visibility-hint {
  font-size: 0.7rem;
  color: var(--text-muted);
  margin-top: -0.25rem;
}

/* Search bar */
.search-bar {
  padding: 0 1.25rem;
  padding-top: 0.75rem;
  position: relative;
}

.search-bar input {
  width: 100%;
  padding: 0.75rem 2rem 0.75rem 0.8rem;
  min-height: 48px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 1rem;
  color: var(--text-primary);
  font-family: var(--font-body);
  box-sizing: border-box;
}

.search-bar input:focus {
  outline: none;
  border-color: var(--accent);
}

.search-clear {
  position: absolute;
  right: 1.6rem;
  top: 50%;
  transform: translateY(-50%);
  margin-top: 0.375rem;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 2px;
  display: flex;
}

.search-clear:hover {
  color: var(--text-primary);
}

/* Breadcrumbs */
.breadcrumb-nav {
  display: flex;
  align-items: center;
  gap: 0.15rem;
  min-width: 0;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
}

.breadcrumb-nav::-webkit-scrollbar {
  display: none;
}

.breadcrumb-sep {
  color: var(--text-muted);
  font-size: 0.75rem;
  flex-shrink: 0;
}

.breadcrumb-item {
  background: none;
  border: none;
  color: var(--text-muted);
  font-family: var(--font-body);
  font-size: 0.8rem;
  cursor: pointer;
  padding: 0.15rem 0.3rem;
  border-radius: 3px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.breadcrumb-item:hover {
  color: var(--text-primary);
  background: var(--bg-tertiary);
}

.breadcrumb-item.current {
  color: var(--text-primary);
  font-weight: 600;
  cursor: default;
}

.breadcrumb-item.current:hover {
  background: none;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin-left: auto;
  flex-shrink: 0;
}

.ns-action-btn {
  padding: 0.3rem 0.6rem;
  font-size: 0.7rem;
  background: var(--bg-active);
  color: var(--text-secondary);
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.ns-action-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

/* Action row */
.action-row {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 0.75rem;
}

.create-ns-btn {
  padding: 0.4rem 0.8rem;
  font-size: 0.75rem;
  background: var(--accent2);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-family: var(--font-body);
}

.create-ns-btn:hover {
  filter: brightness(1.1);
}

/* Search results grouped */
.search-results {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.search-group {
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}

.search-group-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.75rem 1rem;
  background: none;
  border: none;
  color: var(--text-primary);
  font-family: var(--font-body);
  font-size: 0.9rem;
  font-weight: 600;
  text-align: left;
  cursor: pointer;
  transition: background 0.1s;
}

.search-group-header:hover {
  background: var(--bg-active);
}

.search-group-icon {
  font-size: 1rem;
  flex-shrink: 0;
}

.search-group-icon-default {
  font-weight: 700;
  color: var(--accent2);
  opacity: 0.7;
}

.search-group-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.search-group-count {
  font-size: 0.75rem;
  color: var(--text-muted);
  font-weight: 400;
  flex-shrink: 0;
}

.search-group-arrow {
  color: var(--text-muted);
  flex-shrink: 0;
}

.search-group-desc {
  padding: 0 1rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin-top: -0.35rem;
  margin-bottom: 0.5rem;
}

.search-group-preview {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  padding: 0 1rem 0.75rem;
}

.search-preview-tag {
  font-size: 0.75rem;
  padding: 0.2rem 0.5rem;
  border-radius: 3px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 160px;
}

.search-preview-tag.ns-tag {
  background: color-mix(in srgb, var(--accent2) 15%, transparent);
  color: var(--accent2);
  font-family: var(--font-code);
}

.search-preview-tag.room-tag {
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  color: var(--accent);
}

.search-preview-more {
  font-size: 0.75rem;
  color: var(--text-muted);
  padding: 0.15rem 0.3rem;
}

/* Search result namespace path */
.card-ns-path {
  font-size: 0.75rem;
  color: var(--accent2);
  font-family: var(--font-code);
}

/* Namespace settings extras */
.ns-settings-divider {
  border: none;
  border-top: 1px solid var(--border);
  margin: 0.5rem 0;
}

.ns-pw-row {
  display: flex;
  gap: 0.5rem;
}

.ns-pw-row input {
  flex: 1;
}

.pw-change-btn {
  padding: 0.4rem 0.8rem;
  background: var(--accent2);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.75rem;
  white-space: nowrap;
}

.ns-danger-zone {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.ns-danger-label {
  color: var(--danger) !important;
}

.ns-delete-btn {
  padding: 0.5rem;
  background: transparent;
  color: var(--danger);
  border: 1px solid var(--danger);
  border-radius: 4px;
  font-size: 0.8rem;
  cursor: pointer;
}

.ns-delete-btn:hover {
  background: var(--danger);
  color: white;
}

.ns-delete-warning {
  font-size: 0.75rem;
  color: var(--danger);
  line-height: 1.3;
}

.browse-explainer {
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin-bottom: 1rem;
  line-height: 1.4;
}

.private-ns-hint {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-bottom: 0.5rem;
}

.ns-delete-confirm {
  padding: 0.5rem 1rem;
  background: var(--danger);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.8rem;
}
</style>
