<script setup lang="ts">
defineProps<{
  message: string
  hint?: string
  small?: boolean
}>()
</script>

<template>
  <div :class="small ? 'empty-state-small' : 'empty-state'">
    <div v-if="$slots.icon && !small" class="empty-state-icon">
      <slot name="icon" />
    </div>
    <div class="empty-state-text">{{ message }}</div>
    <div v-if="hint" class="empty-state-hint">{{ hint }}</div>
  </div>
</template>
