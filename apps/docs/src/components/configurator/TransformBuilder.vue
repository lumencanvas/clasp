<script setup>
const props = defineProps({
  modelValue: { type: Array, default: () => [] }
})
const emit = defineEmits(['update:modelValue'])

function addTransform() {
  emit('update:modelValue', [...props.modelValue, { path: '', redactFields: '' }])
}

function updateTransform(index, field, value) {
  const next = [...props.modelValue]
  next[index] = { ...next[index], [field]: value }
  emit('update:modelValue', next)
}

function removeTransform(index) {
  emit('update:modelValue', props.modelValue.filter((_, i) => i !== index))
}
</script>

<template>
  <div class="transform-builder">
    <div v-for="(t, i) in modelValue" :key="i" class="transform-row">
      <input
        class="transform-path"
        :value="t.path"
        placeholder="/path/pattern/**"
        @input="updateTransform(i, 'path', $event.target.value)"
      />
      <input
        class="transform-fields"
        :value="t.redactFields"
        placeholder="field1, field2 (comma-separated)"
        @input="updateTransform(i, 'redactFields', $event.target.value)"
      />
      <button class="transform-remove" @click="removeTransform(i)">&times;</button>
    </div>
    <button class="add-btn" @click="addTransform">+ Add Transform</button>
  </div>
</template>

<style scoped>
.transform-builder {
  margin-top: 0.5rem;
}

.transform-row {
  display: flex;
  gap: 0.4rem;
  align-items: center;
  margin-bottom: 0.3rem;
}

.transform-path,
.transform-fields {
  flex: 1;
  padding: 0.35rem 0.5rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
}

.transform-path:focus,
.transform-fields:focus {
  outline: none;
  border-color: var(--accent);
}

.transform-remove {
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
  flex-shrink: 0;
}

.transform-remove:hover {
  color: var(--accent);
  border-color: var(--accent);
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
