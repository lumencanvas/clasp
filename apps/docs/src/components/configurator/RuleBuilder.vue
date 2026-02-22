<script setup>
import RuleCard from './RuleCard.vue'

const props = defineProps({
  modelValue: { type: Array, default: () => [] }
})
const emit = defineEmits(['update:modelValue'])

let nextId = 1

function addRule() {
  emit('update:modelValue', [...props.modelValue, {
    id: 'rule_' + nextId++,
    name: '',
    trigger: { type: '' },
    conditions: [],
    actions: [],
    cooldown: 0
  }])
}

function updateRule(index, rule) {
  const next = [...props.modelValue]
  next[index] = rule
  emit('update:modelValue', next)
}

function removeRule(index) {
  emit('update:modelValue', props.modelValue.filter((_, i) => i !== index))
}
</script>

<template>
  <div class="rule-builder">
    <RuleCard
      v-for="(rule, i) in modelValue"
      :key="rule.id || i"
      :rule="rule"
      @update="updateRule(i, $event)"
      @remove="removeRule(i)"
    />
    <button class="add-btn" @click="addRule">+ Add Rule</button>
  </div>
</template>

<style scoped>
.rule-builder {
  margin-top: 0.5rem;
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
