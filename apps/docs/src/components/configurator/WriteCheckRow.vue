<script setup>
const props = defineProps({
  check: { type: Object, required: true },
  captures: { type: Array, default: () => [] }
})
const emit = defineEmits(['update', 'remove'])

const CHECK_TYPES = [
  'segment_equals_session',
  'state_field_equals_session',
  'state_not_null',
  'value_field_equals_session',
  'either_state_not_null',
  'require_value_field',
  'reject_unless_path_matches'
]

function update(field, value) {
  emit('update', { ...props.check, [field]: value })
}

function updateType(type) {
  // Reset fields when type changes
  const base = { type }
  if (type === 'segment_equals_session') base.segment = props.captures[0] || ''
  if (type === 'state_field_equals_session') { base.lookup = ''; base.field = ''; base.allow_if_missing = false }
  if (type === 'state_not_null') base.lookup = ''
  if (type === 'value_field_equals_session') base.field = ''
  if (type === 'either_state_not_null') { base.lookup_a = ''; base.lookup_b = '' }
  if (type === 'require_value_field') base.field = ''
  if (type === 'reject_unless_path_matches') { base.pattern = ''; base.message = '' }
  emit('update', base)
}
</script>

<template>
  <div class="check-row">
    <div class="check-row-top">
      <select class="check-type" :value="check.type" @change="updateType($event.target.value)">
        <option value="">Select check type...</option>
        <option v-for="t in CHECK_TYPES" :key="t" :value="t">{{ t }}</option>
      </select>
      <button class="check-remove" @click="emit('remove')">&times;</button>
    </div>

    <div v-if="check.type === 'segment_equals_session'" class="check-fields">
      <label class="check-field-label">Segment</label>
      <select class="check-field-input" :value="check.segment" @change="update('segment', $event.target.value)">
        <option v-for="c in captures" :key="c" :value="c">{{ c }}</option>
        <option v-if="!captures.length" value="">(no captures in path)</option>
      </select>
    </div>

    <div v-if="check.type === 'state_field_equals_session'" class="check-fields">
      <label class="check-field-label">Lookup path</label>
      <input class="check-field-input" :value="check.lookup" @input="update('lookup', $event.target.value)" placeholder="/path/to/state" />
      <label class="check-field-label">Field</label>
      <input class="check-field-input" :value="check.field" @input="update('field', $event.target.value)" placeholder="userId" />
      <label class="check-field-label check-field-inline">
        <input type="checkbox" :checked="check.allow_if_missing" @change="update('allow_if_missing', $event.target.checked)" />
        Allow if missing
      </label>
    </div>

    <div v-if="check.type === 'state_not_null'" class="check-fields">
      <label class="check-field-label">Lookup path</label>
      <input class="check-field-input" :value="check.lookup" @input="update('lookup', $event.target.value)" placeholder="/path/to/state" />
    </div>

    <div v-if="check.type === 'value_field_equals_session'" class="check-fields">
      <label class="check-field-label">Field</label>
      <input class="check-field-input" :value="check.field" @input="update('field', $event.target.value)" placeholder="userId" />
    </div>

    <div v-if="check.type === 'either_state_not_null'" class="check-fields">
      <label class="check-field-label">Lookup A</label>
      <input class="check-field-input" :value="check.lookup_a" @input="update('lookup_a', $event.target.value)" placeholder="/path/a" />
      <label class="check-field-label">Lookup B</label>
      <input class="check-field-input" :value="check.lookup_b" @input="update('lookup_b', $event.target.value)" placeholder="/path/b" />
    </div>

    <div v-if="check.type === 'require_value_field'" class="check-fields">
      <label class="check-field-label">Field</label>
      <input class="check-field-input" :value="check.field" @input="update('field', $event.target.value)" placeholder="timestamp" />
    </div>

    <div v-if="check.type === 'reject_unless_path_matches'" class="check-fields">
      <label class="check-field-label">Pattern</label>
      <input class="check-field-input" :value="check.pattern" @input="update('pattern', $event.target.value)" placeholder="/allowed/**" />
      <label class="check-field-label">Message</label>
      <input class="check-field-input" :value="check.message" @input="update('message', $event.target.value)" placeholder="Not allowed" />
    </div>
  </div>
</template>

<style scoped>
.check-row {
  border: 1px solid var(--border);
  padding: 0.5rem;
  margin-bottom: 0.3rem;
  background: var(--code-bg);
  border-radius: 3px;
}

.check-row-top {
  display: flex;
  gap: 0.4rem;
  align-items: center;
}

.check-type {
  flex: 1;
  padding: 0.3rem 0.5rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.78rem;
  background: var(--paper);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
}

.check-remove {
  width: 24px;
  height: 24px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  font-size: 1rem;
}

.check-remove:hover {
  color: var(--accent);
}

.check-fields {
  margin-top: 0.4rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.check-field-label {
  font-size: 0.7rem;
  opacity: 0.6;
  letter-spacing: 0.04em;
}

.check-field-input {
  padding: 0.3rem 0.5rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.78rem;
  background: var(--paper);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
}

.check-field-input:focus {
  outline: none;
  border-color: var(--accent);
}

.check-field-inline {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  cursor: pointer;
}

.check-field-inline input[type="checkbox"] {
  cursor: pointer;
}
</style>
