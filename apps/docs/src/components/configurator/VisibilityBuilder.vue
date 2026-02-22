<script setup>
const props = defineProps({
  modelValue: { type: Array, default: () => [] }
})
const emit = defineEmits(['update:modelValue'])

function addRule() {
  emit('update:modelValue', [...props.modelValue, {
    matchMode: 'path',
    path: '',
    pathContains: '',
    visible: true,
    ownerSegment: '',
    publicSub: '',
    lookup: ''
  }])
}

function updateRule(index, field, value) {
  const next = [...props.modelValue]
  next[index] = { ...next[index], [field]: value }
  emit('update:modelValue', next)
}

function removeRule(index) {
  emit('update:modelValue', props.modelValue.filter((_, i) => i !== index))
}
</script>

<template>
  <div class="vis-builder">
    <div v-for="(rule, i) in modelValue" :key="i" class="vis-card">
      <div class="vis-row">
        <select class="vis-input vis-match" :value="rule.matchMode" @change="updateRule(i, 'matchMode', $event.target.value)">
          <option value="path">Path pattern</option>
          <option value="contains">Path contains</option>
          <option value="catchall">Catch-all</option>
        </select>

        <input
          v-if="rule.matchMode === 'path'"
          class="vis-input vis-path"
          :value="rule.path"
          placeholder="/chat/user/**"
          @input="updateRule(i, 'path', $event.target.value)"
        />
        <input
          v-if="rule.matchMode === 'contains'"
          class="vis-input vis-path"
          :value="rule.pathContains"
          placeholder="/private/"
          @input="updateRule(i, 'pathContains', $event.target.value)"
        />

        <select class="vis-input vis-visible" :value="String(rule.visible)" @change="updateRule(i, 'visible', $event.target.value === 'true' ? true : $event.target.value === 'false' ? false : $event.target.value)">
          <option value="true">Visible</option>
          <option value="false">Hidden</option>
          <option value="owner">Owner only</option>
          <option value="require_state_not_null">Require state</option>
        </select>

        <button class="vis-remove" @click="removeRule(i)">&times;</button>
      </div>

      <div v-if="rule.visible === 'owner'" class="vis-fields">
        <label class="vis-field-label">Owner segment</label>
        <input class="vis-field-input" :value="rule.ownerSegment" @input="updateRule(i, 'ownerSegment', $event.target.value)" placeholder="userId" />
        <label class="vis-field-label">Public sub-path</label>
        <input class="vis-field-input" :value="rule.publicSub" @input="updateRule(i, 'publicSub', $event.target.value)" placeholder="profile" />
      </div>

      <div v-if="rule.visible === 'require_state_not_null'" class="vis-fields">
        <label class="vis-field-label">Lookup path</label>
        <input class="vis-field-input" :value="rule.lookup" @input="updateRule(i, 'lookup', $event.target.value)" placeholder="/path/to/check" />
      </div>
    </div>

    <button class="add-btn" @click="addRule">+ Add Visibility Rule</button>
  </div>
</template>

<style scoped>
.vis-builder {
  margin-top: 0.5rem;
}

.vis-card {
  border: 1px solid var(--border);
  padding: 0.5rem;
  margin-bottom: 0.3rem;
  border-radius: 3px;
}

.vis-row {
  display: flex;
  gap: 0.4rem;
  align-items: center;
}

.vis-input {
  padding: 0.35rem 0.5rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.78rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
}

.vis-input:focus {
  outline: none;
  border-color: var(--accent);
}

.vis-match { width: 130px; }
.vis-path { flex: 1; }
.vis-visible { width: 150px; }

.vis-remove {
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

.vis-remove:hover {
  color: var(--accent);
  border-color: var(--accent);
}

.vis-fields {
  margin-top: 0.4rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding-left: 0.5rem;
}

.vis-field-label {
  font-size: 0.7rem;
  opacity: 0.6;
  letter-spacing: 0.04em;
}

.vis-field-input {
  padding: 0.3rem 0.5rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.78rem;
  background: var(--paper);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
}

.vis-field-input:focus {
  outline: none;
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
