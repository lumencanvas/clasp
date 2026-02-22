<script setup>
const props = defineProps({
  modelValue: { type: Boolean, default: false },
  label: { type: String, default: '' }
})
const emit = defineEmits(['update:modelValue'])
</script>

<template>
  <label class="toggle-switch">
    <input
      type="checkbox"
      :checked="modelValue"
      @change="emit('update:modelValue', $event.target.checked)"
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
  gap: 0.5rem;
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
  width: 36px;
  height: 20px;
  background: var(--border);
  border-radius: 10px;
  transition: background 0.2s;
  flex-shrink: 0;
}

.toggle-switch input:checked + .toggle-track {
  background: var(--accent);
}

.toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  background: var(--paper);
  border-radius: 50%;
  transition: transform 0.2s;
}

.toggle-switch input:checked + .toggle-track .toggle-thumb {
  transform: translateX(16px);
}

.toggle-label {
  font-size: 0.82rem;
  letter-spacing: 0.04em;
}
</style>
