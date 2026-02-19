<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ROOM_TYPE_INFO } from '../lib/constants.js'
import { formatRelativeTime } from '../lib/utils.js'
import { useNamespaces } from '../composables/useNamespaces.js'
import { useIdentity } from '../composables/useIdentity.js'
import NamespaceCard from './NamespaceCard.vue'

const props = defineProps({
  rooms: { type: Array, default: () => [] },
  joinedRoomIds: { type: Set, default: () => new Set() },
})

const emit = defineEmits(['join', 'close'])

const { userId } = useIdentity()

const {
  discoveredNamespaces,
  subscribedNamespaces,
  namespaceTree,
  discoverTopLevelNamespaces,
  discoverChildNamespaces,
  pinNamespace,
  unpinNamespace,
  lookupNamespace,
  unlockNamespace,
  namespacedRoomIds,
  updateNamespaceMeta,
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

  // Subscribe to child namespaces
  if (childUnsub) childUnsub()
  childUnsub = discoverChildNamespaces(nsPath)
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

function handlePinNs(nsPath) {
  pinNamespace(nsPath)
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
    nsLookupError.value = 'Namespace not found'
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
          <h3>{{ currentView === 'top' ? 'Browse Public Channels' : currentView }}</h3>
          <button
            v-if="currentView !== 'top' && isNsCreator"
            class="ns-settings-btn"
            title="Namespace settings"
            @click="openNsSettings"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
            </svg>
          </button>
          <button
            v-if="currentView !== 'top'"
            :class="['pin-toggle', { pinned: isNsPinned(currentView) }]"
            @click="isNsPinned(currentView) ? handleUnpinNs(currentView) : handlePinNs(currentView)"
          >
            {{ isNsPinned(currentView) ? 'Pinned' : 'Pin' }}
          </button>
        </div>
        <button class="close-btn" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div class="dialog-body">
        <!-- TOP LEVEL VIEW -->
        <template v-if="currentView === 'top'">
          <!-- Namespaces section -->
          <div v-if="namespaceList.length" class="browse-section">
            <div class="browse-label">Namespaces</div>
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
            <div class="browse-label">Join Private Namespace</div>
            <form class="ns-lookup-row" @submit.prevent="handleLookupNs">
              <input
                v-model="nsPathInput"
                type="text"
                placeholder="namespace path (e.g. secret-club)"
                autocomplete="off"
              />
              <button type="submit" :disabled="!nsPathInput.trim()">Unlock</button>
            </form>
            <p v-if="nsLookupError" class="lookup-error">{{ nsLookupError }}</p>
          </div>

          <!-- Uncategorized rooms -->
          <div v-if="uncategorizedRooms.length" class="browse-section">
            <div class="browse-label">Uncategorized Rooms</div>
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
                >
                  {{ joinedRoomIds.has(room.id) ? 'Joined' : 'Join' }}
                </button>
              </div>
            </div>
          </div>

          <div v-if="!namespaceList.length && !uncategorizedRooms.length" class="empty">
            <p>No public channels found</p>
            <span>Create one and make it public!</span>
          </div>
        </template>

        <!-- NAMESPACE DRILL-IN VIEW -->
        <template v-else>
          <p v-if="currentNsMeta?.description" class="ns-description">{{ currentNsMeta.description }}</p>

          <!-- Sub-namespaces -->
          <div v-if="currentNsChildren.length" class="browse-section">
            <div class="browse-label">Sub-namespaces</div>
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
            <div class="browse-label">Rooms in {{ currentView.split('/').pop() }}</div>
            <div v-if="!currentNsRooms.length" class="empty-small">
              <span>No rooms in this namespace yet</span>
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
                >
                  {{ joinedRoomIds.has(room.id) ? 'Joined' : 'Join' }}
                </button>
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
          <div class="password-dialog" style="width: 360px">
            <h4>Namespace Settings</h4>
            <p>{{ currentView }}</p>
            <div class="ns-settings-form">
              <label class="ns-settings-label">Description</label>
              <input v-model="nsEditDesc" type="text" placeholder="Description" />
              <label class="ns-settings-label">Icon</label>
              <input v-model="nsEditIcon" type="text" placeholder="Icon character" maxlength="2" />
              <div class="ns-settings-toggle">
                <label class="ns-settings-label">Public</label>
                <button
                  type="button"
                  :class="['ns-toggle', { active: nsEditPublic }]"
                  @click="nsEditPublic = !nsEditPublic"
                >
                  <span class="ns-toggle-knob"></span>
                </button>
              </div>
              <div class="pw-actions">
                <button class="pw-cancel" @click="showNsSettings = false">Cancel</button>
                <button class="pw-submit" @click="saveNsSettings">Save</button>
              </div>
            </div>
          </div>
        </div>

        <!-- Namespace password prompt overlay -->
        <div v-if="nsPasswordPrompt" class="password-overlay" @click.self="nsPasswordPrompt = null">
          <div class="password-dialog">
            <h4>Unlock Namespace</h4>
            <p>"{{ nsPasswordPrompt.path }}" is password-protected</p>
            <form @submit.prevent="handleNsPasswordSubmit">
              <input
                v-model="nsPasswordInput"
                type="password"
                placeholder="Namespace password"
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
  max-height: 80vh;
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
  font-size: 0.65rem;
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
  font-size: 0.65rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--accent2);
}

.card-time {
  font-size: 0.65rem;
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
  font-size: 0.65rem;
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

.ns-settings-toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.ns-toggle {
  width: 36px;
  height: 20px;
  background: var(--bg-active);
  border: none;
  border-radius: 10px;
  position: relative;
  transition: background 0.2s;
}

.ns-toggle.active {
  background: var(--success);
}

.ns-toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  background: white;
  border-radius: 50%;
  transition: transform 0.2s;
}

.ns-toggle.active .ns-toggle-knob {
  transform: translateX(16px);
}
</style>
