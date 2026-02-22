<script setup>
import ScopeRow from './ScopeRow.vue'

const props = defineProps({
  modelValue: { type: Array, default: () => [] }
})
const emit = defineEmits(['update:modelValue'])

const TEMPLATES = [
  { label: 'User Own Data', scopes: [
    { action: 'read', pattern: '/chat/user/{userId}/**' },
    { action: 'write', pattern: '/chat/user/{userId}/**' }
  ]},
  { label: 'Room Messages', scopes: [
    { action: 'write', pattern: '/chat/room/*/messages/**' },
    { action: 'subscribe', pattern: '/chat/room/**' }
  ]},
  { label: 'Global Read', scopes: [
    { action: 'read', pattern: '/**' },
    { action: 'subscribe', pattern: '/**' }
  ]},
  { label: 'Admin All', scopes: [
    { action: '*', pattern: '/**' }
  ]}
]

function addScope() {
  emit('update:modelValue', [...props.modelValue, { action: 'read', pattern: '' }])
}

function addTemplate(tpl) {
  emit('update:modelValue', [...props.modelValue, ...tpl.scopes.map(s => ({ ...s }))])
}

function updateScope(index, scope) {
  const next = [...props.modelValue]
  next[index] = scope
  emit('update:modelValue', next)
}

function removeScope(index) {
  emit('update:modelValue', props.modelValue.filter((_, i) => i !== index))
}
</script>

<template>
  <div class="scope-builder">
    <div class="scope-templates">
      <span class="scope-templates-label">Quick add:</span>
      <button
        v-for="tpl in TEMPLATES"
        :key="tpl.label"
        class="scope-tpl-btn"
        @click="addTemplate(tpl)"
      >
        {{ tpl.label }}
      </button>
    </div>
    <ScopeRow
      v-for="(scope, i) in modelValue"
      :key="i"
      :scope="scope"
      @update="updateScope(i, $event)"
      @remove="removeScope(i)"
    />
    <button class="add-btn" @click="addScope">+ Add Scope</button>
  </div>
</template>

<style scoped>
.scope-builder {
  margin-top: 0.5rem;
}

.scope-templates {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
  align-items: center;
  margin-bottom: 0.5rem;
}

.scope-templates-label {
  font-size: 0.72rem;
  opacity: 0.5;
  letter-spacing: 0.06em;
  margin-right: 0.25rem;
}

.scope-tpl-btn {
  font-family: 'Space Mono', monospace;
  font-size: 0.7rem;
  padding: 0.2rem 0.5rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  color: var(--accent2);
  cursor: pointer;
  border-radius: 2px;
  letter-spacing: 0.04em;
  transition: border-color 0.15s;
}

.scope-tpl-btn:hover {
  border-color: var(--accent2);
}

.add-btn {
  font-family: 'Space Mono', monospace;
  font-size: 0.75rem;
  padding: 0.3rem 0.75rem;
  background: none;
  border: 1px dashed var(--border);
  color: var(--accent);
  cursor: pointer;
  border-radius: 3px;
  letter-spacing: 0.04em;
  margin-top: 0.4rem;
  transition: border-color 0.15s;
}

.add-btn:hover {
  border-color: var(--accent);
}
</style>
