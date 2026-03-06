<script setup lang="ts">
defineProps<{
  label: string
  hint?: string
}>()
</script>

<template>
  <div class="field-row">
    <label class="field-label">{{ label }}</label>
    <div class="field-control">
      <slot />
    </div>
    <span v-if="hint" class="field-hint">{{ hint }}</span>
  </div>
</template>

<style scoped>
.field-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-xs) var(--space-sm);
  min-height: 28px;
}

.field-label {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.3px;
  color: var(--color-text-muted);
  min-width: 100px;
  flex-shrink: 0;
}

.field-control {
  flex: 1;
  min-width: 0;
}

.field-control :deep(input),
.field-control :deep(select) {
  width: 100%;
}

.field-hint {
  width: 100%;
  font-size: 10px;
  color: var(--stone-400);
  font-family: var(--font-mono);
  padding-left: calc(100px + var(--space-sm));
}
</style>
