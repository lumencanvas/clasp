<script setup>
const props = defineProps({
  rule: { type: Object, required: true }
})
const emit = defineEmits(['update', 'remove'])

const TRIGGER_TYPES = ['on_change', 'on_threshold', 'on_event', 'on_interval']
const OPERATORS = ['eq', 'ne', 'gt', 'gte', 'lt', 'lte']
const ACTION_TYPES = ['set', 'publish', 'set_from_trigger', 'delay']

function update(field, value) {
  emit('update', { ...props.rule, [field]: value })
}

function updateTrigger(field, value) {
  emit('update', { ...props.rule, trigger: { ...props.rule.trigger, [field]: value } })
}

function setTriggerType(type) {
  const base = { type }
  if (type === 'on_change') base.address = ''
  if (type === 'on_threshold') { base.address = ''; base.threshold = 0; base.direction = 'above' }
  if (type === 'on_event') { base.address = ''; base.event = '' }
  if (type === 'on_interval') base.seconds = 60
  emit('update', { ...props.rule, trigger: base })
}

// Conditions
function addCondition() {
  const conds = [...(props.rule.conditions || []), { address: '', operator: 'eq', value: '' }]
  emit('update', { ...props.rule, conditions: conds })
}

function updateCondition(i, field, value) {
  const conds = [...props.rule.conditions]
  conds[i] = { ...conds[i], [field]: value }
  emit('update', { ...props.rule, conditions: conds })
}

function removeCondition(i) {
  emit('update', { ...props.rule, conditions: props.rule.conditions.filter((_, idx) => idx !== i) })
}

// Actions
function addAction() {
  const acts = [...(props.rule.actions || []), { type: 'set', address: '', value: '' }]
  emit('update', { ...props.rule, actions: acts })
}

function updateAction(i, field, value) {
  const acts = [...props.rule.actions]
  acts[i] = { ...acts[i], [field]: value }
  emit('update', { ...props.rule, actions: acts })
}

function removeAction(i) {
  emit('update', { ...props.rule, actions: props.rule.actions.filter((_, idx) => idx !== i) })
}
</script>

<template>
  <div class="rule-card">
    <div class="rule-card-header">
      <input
        class="rule-name"
        :value="rule.name"
        placeholder="Rule name"
        @input="update('name', $event.target.value)"
      />
      <button class="rule-remove" @click="emit('remove')">&times;</button>
    </div>

    <!-- Trigger -->
    <div class="rule-section">
      <div class="rule-section-label">Trigger</div>
      <select class="rule-select" :value="(rule.trigger || {}).type || ''" @change="setTriggerType($event.target.value)">
        <option value="">Select trigger...</option>
        <option v-for="t in TRIGGER_TYPES" :key="t" :value="t">{{ t }}</option>
      </select>

      <template v-if="rule.trigger">
        <div v-if="rule.trigger.type === 'on_change' || rule.trigger.type === 'on_threshold' || rule.trigger.type === 'on_event'" class="rule-trigger-fields">
          <input class="rule-field-input" :value="rule.trigger.address" @input="updateTrigger('address', $event.target.value)" placeholder="Address pattern" />
        </div>
        <div v-if="rule.trigger.type === 'on_threshold'" class="rule-trigger-fields">
          <input class="rule-field-input" type="number" :value="rule.trigger.threshold" @input="updateTrigger('threshold', Number($event.target.value))" placeholder="Threshold" />
          <select class="rule-select-sm" :value="rule.trigger.direction" @change="updateTrigger('direction', $event.target.value)">
            <option value="above">Above</option>
            <option value="below">Below</option>
          </select>
        </div>
        <div v-if="rule.trigger.type === 'on_event'" class="rule-trigger-fields">
          <input class="rule-field-input" :value="rule.trigger.event" @input="updateTrigger('event', $event.target.value)" placeholder="Event name" />
        </div>
        <div v-if="rule.trigger.type === 'on_interval'" class="rule-trigger-fields">
          <input class="rule-field-input" type="number" :value="rule.trigger.seconds" @input="updateTrigger('seconds', Number($event.target.value))" placeholder="Seconds" />
        </div>
      </template>
    </div>

    <!-- Conditions -->
    <div class="rule-section">
      <div class="rule-section-label">Conditions</div>
      <div v-for="(cond, i) in (rule.conditions || [])" :key="'c-' + i" class="rule-cond-row">
        <input class="rule-field-input" :value="cond.address" @input="updateCondition(i, 'address', $event.target.value)" placeholder="Address" />
        <select class="rule-select-sm" :value="cond.operator" @change="updateCondition(i, 'operator', $event.target.value)">
          <option v-for="op in OPERATORS" :key="op" :value="op">{{ op }}</option>
        </select>
        <input class="rule-field-input" :value="cond.value" @input="updateCondition(i, 'value', $event.target.value)" placeholder="Value" />
        <button class="rule-mini-remove" @click="removeCondition(i)">&times;</button>
      </div>
      <button class="add-btn-sm" @click="addCondition">+ Condition</button>
    </div>

    <!-- Actions -->
    <div class="rule-section">
      <div class="rule-section-label">Actions</div>
      <div v-for="(act, i) in (rule.actions || [])" :key="'a-' + i" class="rule-action-row">
        <select class="rule-select-sm" :value="act.type" @change="updateAction(i, 'type', $event.target.value)">
          <option v-for="a in ACTION_TYPES" :key="a" :value="a">{{ a }}</option>
        </select>
        <input class="rule-field-input" :value="act.address" @input="updateAction(i, 'address', $event.target.value)" placeholder="Address" />
        <input v-if="act.type !== 'set_from_trigger'" class="rule-field-input" :value="act.value" @input="updateAction(i, 'value', $event.target.value)" placeholder="Value" />
        <input v-if="act.type === 'set_from_trigger'" class="rule-field-input" :value="act.transform" @input="updateAction(i, 'transform', $event.target.value)" placeholder="Transform expr" />
        <input v-if="act.type === 'delay'" class="rule-field-input rule-field-sm" type="number" :value="act.delayMs" @input="updateAction(i, 'delayMs', Number($event.target.value))" placeholder="ms" />
        <button class="rule-mini-remove" @click="removeAction(i)">&times;</button>
      </div>
      <button class="add-btn-sm" @click="addAction">+ Action</button>
    </div>

    <!-- Cooldown -->
    <div class="rule-section rule-section-inline">
      <label class="rule-section-label">Cooldown (ms)</label>
      <input class="rule-field-input rule-field-sm" type="number" :value="rule.cooldown || 0" @input="update('cooldown', Number($event.target.value))" />
    </div>
  </div>
</template>

<style scoped>
.rule-card {
  border: 1px solid var(--border);
  padding: 0.75rem;
  margin-bottom: 0.5rem;
  border-radius: 3px;
}

.rule-card-header {
  display: flex;
  gap: 0.4rem;
  align-items: center;
  margin-bottom: 0.5rem;
}

.rule-name {
  flex: 1;
  padding: 0.4rem 0.6rem;
  font-family: 'Space Mono', monospace;
  font-size: 0.85rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
  font-weight: 700;
}

.rule-name:focus { outline: none; border-color: var(--accent); }

.rule-remove {
  width: 28px; height: 28px; padding: 0;
  display: flex; align-items: center; justify-content: center;
  background: none; border: 1px solid transparent;
  color: var(--muted); cursor: pointer; font-size: 1.1rem; flex-shrink: 0;
}
.rule-remove:hover { color: var(--accent); border-color: var(--accent); }

.rule-section { margin-top: 0.5rem; }

.rule-section-label {
  font-size: 0.7rem; letter-spacing: 0.1em;
  font-weight: 700; opacity: 0.5; text-transform: uppercase;
  margin-bottom: 0.3rem;
}

.rule-section-inline {
  display: flex; align-items: center; gap: 0.5rem;
}

.rule-section-inline .rule-section-label { margin-bottom: 0; }

.rule-select,
.rule-field-input {
  padding: 0.3rem 0.5rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.78rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
}

.rule-field-input:focus,
.rule-select:focus { outline: none; border-color: var(--accent); }

.rule-select { width: 100%; margin-bottom: 0.3rem; }

.rule-select-sm {
  padding: 0.3rem 0.4rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
  width: 80px;
}

.rule-field-sm { width: 80px; }

.rule-trigger-fields {
  display: flex; gap: 0.4rem; align-items: center;
  margin-top: 0.3rem;
}

.rule-trigger-fields .rule-field-input { flex: 1; }

.rule-cond-row,
.rule-action-row {
  display: flex; gap: 0.4rem; align-items: center;
  margin-bottom: 0.3rem;
}

.rule-cond-row .rule-field-input,
.rule-action-row .rule-field-input { flex: 1; min-width: 0; }

.rule-mini-remove {
  width: 22px; height: 22px; padding: 0;
  display: flex; align-items: center; justify-content: center;
  background: none; border: none;
  color: var(--muted); cursor: pointer; font-size: 0.9rem; flex-shrink: 0;
}
.rule-mini-remove:hover { color: var(--accent); }

.add-btn-sm {
  font-family: 'Space Mono', monospace;
  font-size: 0.7rem; padding: 0.2rem 0.5rem;
  background: none; border: 1px dashed var(--border);
  color: var(--accent); cursor: pointer; border-radius: 2px;
  margin-top: 0.25rem; transition: border-color 0.15s;
}
.add-btn-sm:hover { border-color: var(--accent); }
</style>
