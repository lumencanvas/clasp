<script setup>
import { computed } from 'vue'
import WriteCheckRow from './WriteCheckRow.vue'

const props = defineProps({
  rule: { type: Object, required: true }
})
const emit = defineEmits(['update', 'remove'])

// Extract {name} captures from path pattern
const captures = computed(() => {
  const matches = (props.rule.path || '').match(/\{(\w+)\}/g)
  return matches ? matches.map(m => m.slice(1, -1)) : []
})

function update(field, value) {
  emit('update', { ...props.rule, [field]: value })
}

function addCheck(listKey) {
  const next = { ...props.rule }
  next[listKey] = [...(next[listKey] || []), { type: '' }]
  emit('update', next)
}

function updateCheck(listKey, index, check) {
  const next = { ...props.rule }
  next[listKey] = [...next[listKey]]
  next[listKey][index] = check
  emit('update', next)
}

function removeCheck(listKey, index) {
  const next = { ...props.rule }
  next[listKey] = next[listKey].filter((_, i) => i !== index)
  emit('update', next)
}
</script>

<template>
  <div class="write-rule-card">
    <div class="write-rule-header">
      <input
        class="write-rule-path"
        :value="rule.path"
        placeholder="/chat/room/{roomId}/messages/{msgId}"
        @input="update('path', $event.target.value)"
      />
      <button class="write-rule-remove" @click="emit('remove')">&times;</button>
    </div>

    <div class="write-rule-options">
      <label class="write-rule-option">
        Mode:
        <select :value="rule.mode || 'all'" @change="update('mode', $event.target.value)">
          <option value="all">All checks pass</option>
          <option value="any">Any check passes</option>
        </select>
      </label>
      <label class="write-rule-option">
        <input
          type="checkbox"
          :checked="rule.allowNullWrite"
          @change="update('allowNullWrite', $event.target.checked)"
        />
        Allow null write
      </label>
    </div>

    <div v-if="captures.length" class="write-rule-captures">
      Captures: <code v-for="c in captures" :key="c">{{ '{' + c + '}' }}</code>
    </div>

    <div class="write-rule-checks-section">
      <div class="write-rule-checks-label">Pre-checks (always run)</div>
      <WriteCheckRow
        v-for="(check, i) in (rule.preChecks || [])"
        :key="'pre-' + i"
        :check="check"
        :captures="captures"
        @update="updateCheck('preChecks', i, $event)"
        @remove="removeCheck('preChecks', i)"
      />
      <button class="add-btn-sm" @click="addCheck('preChecks')">+ Pre-check</button>
    </div>

    <div class="write-rule-checks-section">
      <div class="write-rule-checks-label">Checks</div>
      <WriteCheckRow
        v-for="(check, i) in (rule.checks || [])"
        :key="'chk-' + i"
        :check="check"
        :captures="captures"
        @update="updateCheck('checks', i, $event)"
        @remove="removeCheck('checks', i)"
      />
      <button class="add-btn-sm" @click="addCheck('checks')">+ Check</button>
    </div>
  </div>
</template>

<style scoped>
.write-rule-card {
  border: 1px solid var(--border);
  padding: 0.75rem;
  margin-bottom: 0.5rem;
  border-radius: 3px;
}

.write-rule-header {
  display: flex;
  gap: 0.4rem;
  align-items: center;
  margin-bottom: 0.5rem;
}

.write-rule-path {
  flex: 1;
  padding: 0.4rem 0.6rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.82rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
}

.write-rule-path:focus {
  outline: none;
  border-color: var(--accent);
}

.write-rule-remove {
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

.write-rule-remove:hover {
  color: var(--accent);
  border-color: var(--accent);
}

.write-rule-options {
  display: flex;
  gap: 1rem;
  align-items: center;
  margin-bottom: 0.5rem;
  flex-wrap: wrap;
}

.write-rule-option {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.75rem;
  letter-spacing: 0.04em;
  cursor: pointer;
}

.write-rule-option select {
  padding: 0.2rem 0.4rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
}

.write-rule-captures {
  font-size: 0.72rem;
  opacity: 0.6;
  margin-bottom: 0.5rem;
}

.write-rule-captures code {
  font-family: 'JetBrains Mono', monospace;
  background: var(--code-bg);
  padding: 0.1em 0.3em;
  border-radius: 2px;
  font-size: 0.75rem;
  margin: 0 0.15rem;
}

.write-rule-checks-section {
  margin-top: 0.5rem;
}

.write-rule-checks-label {
  font-size: 0.7rem;
  letter-spacing: 0.1em;
  font-weight: 700;
  opacity: 0.5;
  text-transform: uppercase;
  margin-bottom: 0.3rem;
}

.add-btn-sm {
  font-family: 'Space Mono', monospace;
  font-size: 0.7rem;
  padding: 0.2rem 0.5rem;
  background: none;
  border: 1px dashed var(--border);
  color: var(--accent);
  cursor: pointer;
  border-radius: 2px;
  margin-top: 0.25rem;
  transition: border-color 0.15s;
}

.add-btn-sm:hover {
  border-color: var(--accent);
}
</style>
