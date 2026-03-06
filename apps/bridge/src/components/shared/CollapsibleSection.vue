<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  title: string
  defaultOpen?: boolean
}>()

const open = ref(props.defaultOpen ?? false)
</script>

<template>
  <div class="collapsible-section">
    <button type="button" class="collapsible-header" :aria-expanded="open" @click="open = !open">
      <svg :class="{ rotated: open }" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="9 18 15 12 9 6" />
      </svg>
      <span class="collapsible-title">{{ title }}</span>
    </button>
    <div v-show="open" class="collapsible-body">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.collapsible-header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  width: 100%;
  padding: var(--space-sm) 0;
  background: none;
  border: none;
  border-top: 1px dashed var(--stone-300);
  cursor: pointer;
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: var(--color-text-muted);
}

.collapsible-header:hover {
  color: var(--color-text);
}

.collapsible-header svg {
  transition: transform 0.15s;
}

.collapsible-header svg.rotated {
  transform: rotate(90deg);
}

.collapsible-body {
  padding: var(--space-sm) 0;
}
</style>
