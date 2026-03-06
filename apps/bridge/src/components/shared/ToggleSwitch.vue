<script setup lang="ts">
defineProps<{
  modelValue: boolean
  label?: string
}>()

defineEmits<{
  'update:modelValue': [value: boolean]
}>()
</script>

<template>
  <label class="toggle-switch">
    <input
      type="checkbox"
      role="switch"
      :aria-checked="modelValue"
      :checked="modelValue"
      @change="$emit('update:modelValue', ($event.target as HTMLInputElement).checked)"
    />
    <span class="toggle-track">
      <span class="toggle-thumb" />
    </span>
    <span v-if="label" class="toggle-label">{{ label }}</span>
  </label>
</template>

<style scoped>
.toggle-switch {
  display: inline-flex;
  align-items: center;
  gap: var(--space-sm);
  cursor: pointer;
  user-select: none;
}

.toggle-switch input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-track {
  position: relative;
  width: 32px;
  height: 18px;
  background: var(--stone-300);
  border: var(--border-width) solid var(--stone-900);
  transition: background 0.15s;
}

.toggle-switch input:checked + .toggle-track {
  background: var(--color-accent);
}

.toggle-thumb {
  position: absolute;
  top: 1px;
  left: 1px;
  width: 12px;
  height: 12px;
  background: var(--stone-900);
  transition: left 0.15s;
}

.toggle-switch input:checked + .toggle-track .toggle-thumb {
  left: 15px;
}

.toggle-label {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.3px;
  color: var(--color-text-muted);
}
</style>
