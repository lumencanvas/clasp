<script setup lang="ts">
import { ref } from 'vue'
import type { Rule, RuleTrigger, RuleCondition, RuleAction } from '../../lib/types'
import { ruleTriggerTypes, ruleActionTypes, ruleOperators } from '../../lib/constants'
import CollapsibleSection from '../shared/CollapsibleSection.vue'

const dialogRef = ref<HTMLDialogElement | null>(null)
const isEdit = ref(false)

const name = ref('')
const enabled = ref(true)
const trigger = ref<RuleTrigger>({ type: 'on_change', address: '' })
const conditions = ref<RuleCondition[]>([])
const actions = ref<RuleAction[]>([{ type: 'set', address: '', value: 0 }])
const cooldown = ref(0)
const editId = ref('')

const emit = defineEmits<{
  save: [rule: Rule]
}>()

function open(rule?: Rule) {
  if (rule) {
    isEdit.value = true
    editId.value = rule.id
    name.value = rule.name
    enabled.value = rule.enabled
    trigger.value = { ...rule.trigger }
    conditions.value = rule.conditions.map(c => ({ ...c }))
    actions.value = rule.actions.map(a => ({ ...a }))
    cooldown.value = rule.cooldown ?? 0
  } else {
    isEdit.value = false
    editId.value = ''
    name.value = 'New Rule'
    enabled.value = true
    trigger.value = { type: 'on_change', address: '' }
    conditions.value = []
    actions.value = [{ type: 'set', address: '', value: 0 }]
    cooldown.value = 0
  }
  dialogRef.value?.showModal()
}

function close() {
  dialogRef.value?.close()
}

function addCondition() {
  conditions.value.push({ address: '', operator: 'eq', value: 0 })
}

function removeCondition(idx: number) {
  conditions.value.splice(idx, 1)
}

function addAction() {
  actions.value.push({ type: 'set', address: '', value: 0 })
}

function removeAction(idx: number) {
  if (actions.value.length > 1) actions.value.splice(idx, 1)
}

function save() {
  const rule: Rule = {
    id: isEdit.value ? editId.value : Date.now().toString(),
    name: name.value || 'Untitled Rule',
    enabled: enabled.value,
    trigger: { ...trigger.value },
    conditions: conditions.value.map(c => ({ ...c })),
    actions: actions.value.map(a => ({ ...a })),
    cooldown: cooldown.value || undefined,
  }
  emit('save', rule)
  close()
}

defineExpose({ open, close })
</script>

<template>
  <dialog ref="dialogRef" class="modal" @click.self="close">
    <div class="modal-content modal-content--wide">
      <div class="modal-header">
        <span class="modal-title">{{ isEdit ? 'EDIT RULE' : 'NEW RULE' }}</span>
        <button class="modal-close" @click="close">&times;</button>
      </div>
      <form class="rule-form" @submit.prevent="save">
        <div class="form-group">
          <label class="form-label">Rule Name</label>
          <input v-model="name" class="input" placeholder="My Rule" />
        </div>

        <!-- Trigger -->
        <div class="section-label">TRIGGER</div>
        <div class="form-hint" style="margin-bottom: var(--space-xs)">When this condition is met, the rule fires. <a href="https://docs.clasp.to/relay/rules" target="_blank" class="docs-link">Docs</a></div>
        <div class="form-group">
          <label class="form-label">Type</label>
          <select v-model="trigger.type" class="select">
            <option v-for="t in ruleTriggerTypes" :key="t.value" :value="t.value">{{ t.label }}</option>
          </select>
        </div>
        <div v-if="trigger.type === 'on_change' || trigger.type === 'on_threshold'" class="form-group">
          <label class="form-label">Address</label>
          <input v-model="trigger.address" class="input" placeholder="/path/to/signal" />
        </div>
        <div v-if="trigger.type === 'on_threshold'" class="form-row">
          <div class="form-group">
            <label class="form-label">Threshold</label>
            <input v-model.number="trigger.threshold" class="input" type="number" step="any" />
          </div>
          <div class="form-group">
            <label class="form-label">Direction</label>
            <select v-model="trigger.direction" class="select">
              <option value="rising">Rising</option>
              <option value="falling">Falling</option>
              <option value="both">Both</option>
            </select>
          </div>
        </div>
        <div v-if="trigger.type === 'on_event'" class="form-group">
          <label class="form-label">Event Name</label>
          <input v-model="trigger.event" class="input" placeholder="my_event" />
        </div>
        <div v-if="trigger.type === 'on_interval'" class="form-group">
          <label class="form-label">Interval (seconds)</label>
          <input v-model.number="trigger.seconds" class="input" type="number" step="0.1" />
        </div>

        <!-- Conditions -->
        <CollapsibleSection title="CONDITIONS" :default-open="conditions.length > 0">
          <div v-for="(cond, i) in conditions" :key="i" class="condition-row">
            <input v-model="cond.address" class="input input--sm" placeholder="/address" />
            <select v-model="cond.operator" class="select select--sm">
              <option v-for="op in ruleOperators" :key="op.value" :value="op.value">{{ op.label }}</option>
            </select>
            <input v-model="cond.value" class="input input--sm" placeholder="value" />
            <button type="button" class="btn-icon btn-icon--danger" @click="removeCondition(i)">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
            </button>
          </div>
          <button type="button" class="btn btn-secondary btn--sm" @click="addCondition">+ CONDITION</button>
        </CollapsibleSection>

        <!-- Actions -->
        <div class="section-label">ACTIONS</div>
        <div v-for="(action, i) in actions" :key="i" class="action-row">
          <select v-model="action.type" class="select select--sm">
            <option v-for="at in ruleActionTypes" :key="at.value" :value="at.value">{{ at.label }}</option>
          </select>
          <input v-model="action.address" class="input input--sm" placeholder="/target/address" />
          <input
            v-if="action.type === 'set' || action.type === 'publish'"
            v-model="action.value"
            class="input input--sm"
            placeholder="value"
          />
          <input
            v-if="action.type === 'delay'"
            v-model.number="action.delayMs"
            class="input input--sm"
            type="number"
            placeholder="ms"
          />
          <button
            v-if="actions.length > 1"
            type="button"
            class="btn-icon btn-icon--danger"
            @click="removeAction(i)"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
          </button>
        </div>
        <button type="button" class="btn btn-secondary btn--sm" @click="addAction">+ ACTION</button>

        <!-- Cooldown -->
        <div class="form-group" style="margin-top: var(--space-md)">
          <label class="form-label">Cooldown (ms)</label>
          <input v-model.number="cooldown" class="input" type="number" placeholder="0" />
          <div class="form-hint">Minimum time between rule activations. 0 = no limit.</div>
        </div>

        <div class="modal-actions">
          <button type="button" class="btn btn-secondary" @click="close">CANCEL</button>
          <button type="submit" class="btn btn-primary">{{ isEdit ? 'SAVE' : 'CREATE' }}</button>
        </div>
      </form>
    </div>
  </dialog>
</template>

<style scoped>
.modal-content--wide {
  max-width: 560px;
}

.rule-form {
  max-height: 70vh;
  overflow-y: auto;
  padding-right: var(--space-xs);
}

.section-label {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  color: var(--color-text-muted);
  margin-top: var(--space-md);
  margin-bottom: var(--space-xs);
  border-top: 1px dashed var(--stone-300);
  padding-top: var(--space-sm);
}

.condition-row,
.action-row {
  display: flex;
  gap: var(--space-xs);
  align-items: center;
  margin-bottom: var(--space-xs);
}

.input--sm {
  font-size: 11px;
  padding: 4px 6px;
  min-width: 0;
}

.select--sm {
  font-size: 11px;
  padding: 4px 6px;
  min-width: 70px;
}

.btn--sm {
  font-size: 10px;
  padding: 3px 8px;
  margin-top: var(--space-xs);
}

.btn-icon {
  background: none;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: var(--color-text-muted);
  display: flex;
  flex-shrink: 0;
}

.btn-icon:hover { color: var(--color-text); }
.btn-icon--danger:hover { color: var(--color-error); }

.form-hint {
  font-family: var(--font-mono);
  font-size: 9px;
  color: var(--color-text-muted);
  margin-top: 2px;
}

.docs-link {
  color: var(--color-accent);
  text-decoration: none;
  font-weight: 600;
}

.docs-link:hover {
  text-decoration: underline;
}
</style>
