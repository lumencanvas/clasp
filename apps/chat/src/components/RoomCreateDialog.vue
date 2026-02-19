<script setup>
import { ref, computed } from 'vue'
import { ROOM_TYPES } from '../lib/constants.js'
import { useNamespaces } from '../composables/useNamespaces.js'

const emit = defineEmits(['create', 'close'])

const { subscribedNamespaces, unlockedNamespaces, namespaceTree } = useNamespaces()

const name = ref('')
const type = ref(ROOM_TYPES.TEXT)
const isPublic = ref(true)
const encrypted = ref(false)
const hasPassword = ref(false)
const password = ref('')

// Namespace fields
const namespace = ref('')
const showNsDropdown = ref(false)
const isNewNamespace = ref(false)
const newNsPrivate = ref(false)
const newNsPassword = ref('')

// Available namespaces for autocomplete (pinned + unlocked)
const availableNamespaces = computed(() => {
  const all = new Set([...subscribedNamespaces.value, ...unlockedNamespaces.value])
  return [...all].sort()
})

const filteredNamespaces = computed(() => {
  const q = namespace.value.toLowerCase().trim()
  if (!q) return availableNamespaces.value
  return availableNamespaces.value.filter(ns => ns.toLowerCase().includes(q))
})

function selectNamespace(ns) {
  namespace.value = ns
  showNsDropdown.value = false
  isNewNamespace.value = false
}

function handleNsInput() {
  showNsDropdown.value = true
  // Check if typed value matches any existing namespace
  const trimmed = namespace.value.trim()
  isNewNamespace.value = trimmed.length > 0 && !availableNamespaces.value.includes(trimmed)
}

function handleNsBlur() {
  // Delay to allow click on dropdown items
  setTimeout(() => { showNsDropdown.value = false }, 150)
}

function handleCreate() {
  if (!name.value.trim()) return
  emit('create', {
    name: name.value.trim(),
    type: type.value,
    isPublic: isPublic.value,
    encrypted: encrypted.value,
    password: hasPassword.value ? (password.value || null) : null,
    namespace: namespace.value.trim() || null,
    newNamespace: isNewNamespace.value ? {
      isPublic: !newNsPrivate.value,
      password: newNsPassword.value || null,
    } : null,
  })
  name.value = ''
  password.value = ''
  namespace.value = ''
  encrypted.value = false
}
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('close')" @keydown.escape="emit('close')">
    <div class="dialog">
      <div class="dialog-header">
        <h3>Create Channel</h3>
        <button class="close-btn" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <form class="dialog-body" @submit.prevent="handleCreate">
        <div class="field">
          <label>Channel Name</label>
          <input
            v-model="name"
            type="text"
            placeholder="general"
            autocomplete="off"
            maxlength="32"
            autofocus
          />
        </div>

        <div class="field">
          <label>Channel Type</label>
          <div class="type-selector">
            <button
              type="button"
              :class="['type-btn', { active: type === ROOM_TYPES.TEXT }]"
              @click="type = ROOM_TYPES.TEXT"
            >
              <span class="type-icon">#</span>
              Text
            </button>
            <button
              type="button"
              :class="['type-btn', { active: type === ROOM_TYPES.VIDEO }]"
              @click="type = ROOM_TYPES.VIDEO"
            >
              <svg class="type-icon-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polygon points="23 7 16 12 23 17 23 7"/>
                <rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
              </svg>
              Video
            </button>
            <button
              type="button"
              :class="['type-btn', { active: type === ROOM_TYPES.COMBO }]"
              @click="type = ROOM_TYPES.COMBO"
            >
              <svg class="type-icon-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
                <line x1="3" y1="9" x2="21" y2="9"/>
                <line x1="9" y1="21" x2="9" y2="9"/>
              </svg>
              Combo
            </button>
          </div>
        </div>

        <!-- Namespace picker -->
        <div class="field">
          <label>Namespace <span class="optional">(optional)</span></label>
          <div class="ns-input-wrap">
            <input
              v-model="namespace"
              type="text"
              placeholder="e.g. gaming, dev/rust"
              autocomplete="off"
              @input="handleNsInput"
              @focus="showNsDropdown = true"
              @blur="handleNsBlur"
            />
            <div v-if="showNsDropdown && filteredNamespaces.length" class="ns-dropdown">
              <button
                v-for="ns in filteredNamespaces"
                :key="ns"
                type="button"
                class="ns-option"
                @mousedown.prevent="selectNamespace(ns)"
              >
                {{ ns }}
              </button>
            </div>
          </div>
          <span v-if="isNewNamespace" class="ns-new-hint">New namespace will be created</span>
        </div>

        <!-- New namespace options -->
        <div v-if="isNewNamespace" class="field ns-new-options">
          <div class="visibility-field">
            <label>Visibility</label>
            <div class="segmented-control">
              <button
                type="button"
                :class="['seg-btn', { active: !newNsPrivate }]"
                @click="newNsPrivate = false"
              >Public</button>
              <button
                type="button"
                :class="['seg-btn', { active: newNsPrivate }]"
                @click="newNsPrivate = true"
              >Private</button>
            </div>
            <span class="visibility-hint">{{ newNsPrivate ? 'Accessed by direct link only' : 'Appears in discovery' }}</span>
          </div>
          <div v-if="newNsPrivate" class="ns-pw-field">
            <input
              v-model="newNsPassword"
              type="password"
              placeholder="Namespace password (optional)"
              autocomplete="off"
            />
          </div>
        </div>

        <div class="field toggle-field">
          <label>Public</label>
          <button
            type="button"
            :class="['toggle', { active: isPublic }]"
            @click="isPublic = !isPublic"
          >
            <span class="toggle-knob"></span>
          </button>
          <span class="toggle-hint">{{ isPublic ? 'Visible in Browse' : 'Invite-only' }}</span>
        </div>

        <div class="field toggle-field">
          <label>E2E Encryption</label>
          <button
            type="button"
            :class="['toggle', { active: encrypted }]"
            @click="encrypted = !encrypted"
          >
            <span class="toggle-knob"></span>
          </button>
          <span class="toggle-hint">{{ encrypted ? 'Messages encrypted end-to-end' : 'Off' }}</span>
        </div>

        <div class="field toggle-field">
          <label>Room Password</label>
          <button
            type="button"
            :class="['toggle', { active: hasPassword }]"
            @click="hasPassword = !hasPassword"
          >
            <span class="toggle-knob"></span>
          </button>
          <span class="toggle-hint">{{ hasPassword ? 'Password required to join' : 'Off' }}</span>
        </div>

        <div v-if="hasPassword" class="field">
          <input
            v-model="password"
            type="password"
            placeholder="Enter room password"
            autocomplete="off"
          />
        </div>

        <button type="submit" class="create-btn" :disabled="!name.trim()">
          Create Channel
        </button>
      </form>
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
  max-width: 420px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border);
}

.dialog-header h3 {
  font-size: 1rem;
  letter-spacing: 0.06em;
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
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  max-height: 70dvh;
  overflow-y: auto;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field label {
  font-size: 0.7rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-secondary);
}

.field input {
  padding: 0.75rem 1rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.9rem;
  transition: border-color 0.15s;
}

.field input:focus {
  outline: none;
  border-color: var(--accent);
}

.optional {
  text-transform: none;
  letter-spacing: normal;
  color: var(--text-muted);
}

.type-selector {
  display: flex;
  gap: 0.5rem;
}

.type-btn {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.4rem;
  padding: 0.75rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 0.75rem;
  transition: all 0.15s;
}

.type-btn:hover {
  border-color: var(--text-muted);
  color: var(--text-primary);
}

.type-btn.active {
  border-color: var(--accent);
  color: var(--accent);
  background: rgba(230,57,70,0.08);
}

.type-icon {
  font-size: 1.2rem;
  font-weight: 700;
}

.type-icon-svg {
  width: 20px;
  height: 20px;
}

/* Namespace input */
.ns-input-wrap {
  position: relative;
}

.ns-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 0 0 4px 4px;
  max-height: 140px;
  overflow-y: auto;
  z-index: 10;
}

.ns-option {
  display: block;
  width: 100%;
  padding: 0.5rem 0.75rem;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 0.8rem;
  text-align: left;
}

.ns-option:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.ns-new-hint {
  font-size: 0.7rem;
  color: var(--accent3);
}

.ns-new-options {
  background: var(--bg-tertiary);
  border-radius: 4px;
  padding: 0.75rem;
  gap: 0.75rem;
}

.ns-pw-field {
  margin-top: 0.25rem;
}

.ns-pw-field input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 0.8rem;
  box-sizing: border-box;
}

.ns-pw-field input:focus {
  outline: none;
  border-color: var(--accent);
}

/* Toggle */
.toggle-field {
  flex-direction: row;
  align-items: center;
}

.toggle {
  width: 40px;
  height: 22px;
  background: var(--bg-active);
  border: none;
  border-radius: 11px;
  position: relative;
  transition: background 0.2s;
  flex-shrink: 0;
}

.toggle.active {
  background: var(--success);
}

.toggle-knob {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 16px;
  height: 16px;
  background: white;
  border-radius: 50%;
  transition: transform 0.2s;
}

.toggle.active .toggle-knob {
  transform: translateX(18px);
}

.toggle-hint {
  font-size: 0.75rem;
  color: var(--text-muted);
}

/* Segmented control for visibility */
.visibility-field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.segmented-control {
  display: flex;
  background: var(--bg-active);
  border-radius: 6px;
  padding: 2px;
  gap: 2px;
}

.seg-btn {
  flex: 1;
  padding: 0.4rem 0.75rem;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-muted);
  font-family: var(--font-body);
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.seg-btn.active {
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-weight: 600;
}

.seg-btn:hover:not(.active) {
  color: var(--text-secondary);
}

.visibility-hint {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.create-btn {
  min-height: 44px;
  padding: 0.75rem;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.85rem;
  letter-spacing: 0.06em;
  transition: opacity 0.15s;
}

.create-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.create-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
