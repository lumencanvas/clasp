<script setup>
import { ref, computed } from 'vue'

const emit = defineEmits(['create', 'close'])

const address = ref('')
const description = ref('')
const icon = ref('')
const isPrivate = ref(false)
const password = ref('')

const fullPath = computed(() => {
  // Sanitize each segment: lowercase, replace spaces with dashes, strip invalid chars
  return address.value
    .trim()
    .split('/')
    .map(s => s.trim().toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9_-]/g, ''))
    .filter(Boolean)
    .join('/')
})

const pathSegments = computed(() => {
  if (!fullPath.value) return []
  return fullPath.value.split('/')
})

function handleCreate() {
  if (!fullPath.value) return
  emit('create', {
    path: fullPath.value,
    description: description.value.trim(),
    icon: icon.value.trim(),
    isPublic: !isPrivate.value,
    password: isPrivate.value && password.value ? password.value : null,
  })
}
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('close')" @keydown.escape="emit('close')" tabindex="-1">
    <div class="ns-create-dialog">
      <div class="ns-create-header">
        <h3>Create Channel Group</h3>
        <button class="close-btn" @click="emit('close')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>

      <form class="ns-create-form" @submit.prevent="handleCreate">
        <p class="ns-create-explainer">Groups organize related channels together. Use <code>/</code> to create nested groups.</p>
        <div class="field">
          <label>Group Address</label>
          <input v-model="address" type="text" placeholder="gaming/rust" autocomplete="off" autofocus />
          <span class="hint">Use <code>/</code> to nest groups: <code>dev/rust</code> creates "rust" inside "dev"</span>
        </div>

        <div v-if="pathSegments.length > 1" class="path-breakdown">
          <span
            v-for="(seg, i) in pathSegments"
            :key="i"
            class="path-segment"
          ><span v-if="i > 0" class="path-sep"> &rsaquo; </span><span :class="{ 'path-leaf': i === pathSegments.length - 1 }">{{ seg }}</span></span>
        </div>
        <div v-else-if="fullPath" class="path-breakdown">
          <span class="path-segment path-leaf">{{ fullPath }}</span>
        </div>

        <div class="field">
          <label>Description</label>
          <input v-model="description" type="text" placeholder="What is this group for?" autocomplete="off" />
        </div>

        <div class="field">
          <label>Icon</label>
          <input v-model="icon" type="text" placeholder="Icon character" maxlength="2" autocomplete="off" />
        </div>

        <div class="field">
          <label>Visibility</label>
          <div class="segmented-control">
            <button type="button" :class="['seg-btn', { active: !isPrivate }]" @click="isPrivate = false">Public</button>
            <button type="button" :class="['seg-btn', { active: isPrivate }]" @click="isPrivate = true">Private</button>
          </div>
          <span class="hint">{{ isPrivate ? 'Only people who know the name can find it' : 'Anyone can find and browse this group' }}</span>
        </div>

        <div v-if="isPrivate" class="field">
          <label>Password (optional)</label>
          <input v-model="password" type="password" placeholder="Require a password to join" autocomplete="off" />
        </div>

        <button type="submit" class="create-btn" :disabled="!fullPath">Create Group</button>
      </form>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.ns-create-dialog {
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 12px;
  width: min(420px, 90vw);
  max-height: 85dvh;
  overflow-y: auto;
}

.ns-create-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border);
}

.ns-create-header h3 {
  font-family: var(--font-heading);
  font-size: 1rem;
  margin: 0;
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
}

.close-btn:hover {
  color: var(--text-primary);
}

.close-btn svg {
  width: 18px;
  height: 18px;
}

.ns-create-form {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 1.25rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.field label {
  font-size: 0.75rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.field input {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.5rem 0.75rem;
  color: var(--text-primary);
  font-family: var(--font-body);
  font-size: 0.85rem;
}

.field input:focus {
  outline: none;
  border-color: var(--accent);
}

.path-breakdown {
  font-size: 0.8rem;
  color: var(--text-secondary);
  background: var(--bg-active);
  padding: 0.45rem 0.6rem;
  border-radius: 4px;
  font-family: var(--font-code);
}

.path-sep {
  color: var(--text-muted);
  margin: 0 0.1rem;
}

.path-leaf {
  color: var(--accent);
  font-weight: 600;
}

.hint code {
  font-family: var(--font-code);
  font-size: 0.7rem;
  background: var(--bg-active);
  padding: 0.1rem 0.3rem;
  border-radius: 3px;
}

.ns-create-explainer code {
  font-family: var(--font-code);
  font-size: 0.75rem;
  background: var(--bg-active);
  padding: 0.1rem 0.3rem;
  border-radius: 3px;
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

.hint {
  font-size: 0.7rem;
  color: var(--text-muted);
}

.ns-create-explainer {
  font-size: 0.8rem;
  color: var(--text-secondary);
  line-height: 1.4;
  margin-bottom: 0.25rem;
}

.create-btn {
  min-height: 40px;
  padding: 0.6rem;
  background: var(--accent);
  color: var(--bg-primary);
  border: none;
  border-radius: 6px;
  font-family: var(--font-body);
  font-weight: 600;
  font-size: 0.85rem;
  cursor: pointer;
  margin-top: 0.5rem;
}

.create-btn:hover:not(:disabled) {
  filter: brightness(1.1);
}

.create-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
