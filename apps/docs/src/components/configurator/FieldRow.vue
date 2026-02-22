<script setup>
const props = defineProps({
  label: { type: String, required: true },
  hint: { type: String, default: '' },
  type: { type: String, default: 'text' },
  modelValue: { type: [String, Number], default: '' },
  placeholder: { type: String, default: '' },
  min: { type: Number, default: undefined },
  max: { type: Number, default: undefined }
})
const emit = defineEmits(['update:modelValue'])

function onInput(e) {
  const val = props.type === 'number' ? Number(e.target.value) : e.target.value
  emit('update:modelValue', val)
}
</script>

<template>
  <div class="field-row">
    <label class="field-label">{{ label }}</label>
    <div class="field-input-wrap">
      <slot>
        <input
          class="field-input"
          :type="type"
          :value="modelValue"
          :placeholder="placeholder"
          :min="min"
          :max="max"
          @input="onInput"
        />
      </slot>
    </div>
    <span v-if="hint" class="field-hint">{{ hint }}</span>
  </div>
</template>

<style scoped>
.field-row {
  display: grid;
  grid-template-columns: 160px 1fr auto;
  align-items: center;
  gap: 0.75rem;
  padding: 0.35rem 0;
}

.field-label {
  font-size: 0.78rem;
  letter-spacing: 0.06em;
  opacity: 0.8;
  white-space: nowrap;
}

.field-input-wrap {
  min-width: 0;
}

.field-input {
  width: 100%;
  padding: 0.4rem 0.6rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.82rem;
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 3px;
  color: var(--ink);
  transition: border-color 0.15s;
}

.field-input:focus {
  outline: none;
  border-color: var(--accent);
}

.field-input::placeholder {
  opacity: 0.35;
}

.field-hint {
  font-size: 0.7rem;
  opacity: 0.4;
  white-space: nowrap;
  font-family: 'JetBrains Mono', monospace;
}

@media (max-width: 600px) {
  .field-row {
    grid-template-columns: 1fr;
    gap: 0.25rem;
  }
  .field-hint {
    display: none;
  }
}
</style>
