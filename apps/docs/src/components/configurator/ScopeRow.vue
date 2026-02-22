<script setup>
const props = defineProps({
  scope: { type: Object, required: true }
})
const emit = defineEmits(['update', 'remove'])

const ACTIONS = ['read', 'write', 'subscribe', 'emit', 'admin', '*']

function update(field, value) {
  emit('update', { ...props.scope, [field]: value })
}
</script>

<template>
  <div class="scope-row">
    <select
      class="scope-action"
      :value="scope.action"
      @change="update('action', $event.target.value)"
    >
      <option v-for="a in ACTIONS" :key="a" :value="a">{{ a }}</option>
    </select>
    <input
      class="scope-pattern"
      :value="scope.pattern"
      placeholder="/path/**"
      @input="update('pattern', $event.target.value)"
    />
    <button class="scope-remove" @click="emit('remove')" title="Remove scope">&times;</button>
  </div>
</template>

<style scoped>
.scope-row {
  display: flex;
  gap: 0.4rem;
  align-items: center;
  margin-bottom: 0.3rem;
}

.scope-action {
  width: 110px;
  padding: 0.35rem 0.5rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
  cursor: pointer;
}

.scope-pattern {
  flex: 1;
  padding: 0.35rem 0.5rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
}

.scope-pattern:focus,
.scope-action:focus {
  outline: none;
  border-color: var(--accent);
}

.scope-remove {
  width: 28px;
  height: 28px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: 1px solid transparent;
  color: var(--muted);
  cursor: pointer;
  font-size: 1.1rem;
  border-radius: 3px;
  flex-shrink: 0;
}

.scope-remove:hover {
  color: var(--accent);
  border-color: var(--accent);
}
</style>
