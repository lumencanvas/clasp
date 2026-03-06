<script setup lang="ts">
import { ref } from 'vue'
import { useTokens } from '../../composables/useTokens'

const { tokens, create, remove } = useTokens()

const newTokenName = ref('')
const selectedScopes = ref<string[]>([])
const copiedId = ref<string | null>(null)

const availableScopes = [
  { value: 'read:*', label: 'Read All' },
  { value: 'write:*', label: 'Write All' },
  { value: 'subscribe:*', label: 'Subscribe All' },
  { value: 'emit:*', label: 'Emit All' },
  { value: 'admin:*', label: 'Admin All' },
  { value: 'read:/chat/*', label: 'Read Chat' },
  { value: 'write:/chat/*', label: 'Write Chat' },
  { value: 'read:/bridge/*', label: 'Read Bridge' },
  { value: 'write:/bridge/*', label: 'Write Bridge' },
]

function toggleScope(scope: string) {
  const idx = selectedScopes.value.indexOf(scope)
  if (idx >= 0) {
    selectedScopes.value.splice(idx, 1)
  } else {
    selectedScopes.value.push(scope)
  }
}

function handleCreate() {
  if (!newTokenName.value.trim()) return
  create(newTokenName.value.trim(), '*', [...selectedScopes.value])
  newTokenName.value = ''
  selectedScopes.value = []
}

function handleDelete(id: string) {
  remove(id)
}

async function copyToken(token: string, id: string) {
  try {
    await navigator.clipboard.writeText(token)
    copiedId.value = id
    setTimeout(() => { copiedId.value = null }, 2000)
  } catch {
    // Clipboard unavailable
  }
}

function maskToken(token: string): string {
  if (token.length <= 10) return token
  return token.slice(0, 8) + '...' + token.slice(-4)
}
</script>

<template>
  <div class="token-manager">
    <!-- Token list -->
    <div class="token-list">
      <div v-if="tokens.length === 0" class="token-empty">
        No tokens created yet.
      </div>
      <div v-for="token in tokens" :key="token.id" class="token-item">
        <div class="token-info">
          <span class="token-name">{{ token.name }}</span>
          <code class="token-value">{{ maskToken(token.token) }}</code>
          <span class="token-scopes">{{ token.scopes.join(', ') }}</span>
        </div>
        <div class="token-actions">
          <button
            class="btn btn-sm"
            @click="copyToken(token.token, token.id)"
          >
            {{ copiedId === token.id ? 'Copied' : 'Copy' }}
          </button>
          <button
            class="btn btn-sm btn-danger"
            @click="handleDelete(token.id)"
          >
            Delete
          </button>
        </div>
      </div>
    </div>

    <!-- Create form -->
    <div class="token-create">
      <div class="form-group">
        <label class="form-label">Token Name</label>
        <input
          class="input"
          type="text"
          placeholder="My API Token"
          v-model="newTokenName"
          @keyup.enter="handleCreate"
        />
      </div>
      <div class="form-group">
        <label class="form-label">Scopes</label>
        <div class="scope-grid">
          <label
            v-for="scope in availableScopes"
            :key="scope.value"
            class="scope-checkbox"
          >
            <input
              type="checkbox"
              :checked="selectedScopes.includes(scope.value)"
              @change="toggleScope(scope.value)"
            />
            {{ scope.label }}
          </label>
        </div>
      </div>
      <button
        class="btn btn-primary"
        :disabled="!newTokenName.trim()"
        @click="handleCreate"
      >
        Create Token
      </button>
    </div>
  </div>
</template>

<style scoped>
.token-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  margin-bottom: var(--space-md);
}

.token-empty {
  color: var(--color-text-muted);
  font-size: 12px;
  font-style: italic;
  padding: var(--space-sm) 0;
}

.token-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-sm);
  padding: var(--space-sm);
  background: var(--color-bg-secondary, rgba(255, 255, 255, 0.03));
  border-radius: var(--radius-sm, 4px);
}

.token-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.token-name {
  font-weight: 600;
  font-size: 13px;
}

.token-value {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--color-text-muted);
}

.token-scopes {
  font-size: 10px;
  color: var(--color-text-muted);
}

.token-actions {
  display: flex;
  gap: var(--space-xs);
  flex-shrink: 0;
}

.token-create {
  border-top: 1px dashed var(--stone-300, #444);
  padding-top: var(--space-md);
}

.scope-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: var(--space-xs);
}

.scope-checkbox {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  font-size: 12px;
  cursor: pointer;
}
</style>
