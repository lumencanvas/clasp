<script setup>
import WriteRuleCard from './WriteRuleCard.vue'

const props = defineProps({
  modelValue: { type: Array, default: () => [] }
})
const emit = defineEmits(['update:modelValue'])

function addRule() {
  emit('update:modelValue', [...props.modelValue, {
    path: '',
    mode: 'all',
    allowNullWrite: false,
    preChecks: [],
    checks: []
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
  <div class="write-rule-builder">
    <WriteRuleCard
      v-for="(rule, i) in modelValue"
      :key="i"
      :rule="rule"
      @update="updateRule(i, $event)"
      @remove="removeRule(i)"
    />
    <button class="add-btn" @click="addRule">+ Add Write Rule</button>
  </div>
</template>

<style scoped>
.write-rule-builder {
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
