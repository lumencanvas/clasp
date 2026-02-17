<script setup>
defineProps({
  text: { type: String, required: true },
})
</script>

<template>
  <div class="system-message">
    <span class="system-dot"></span>
    {{ text }}
  </div>
</template>

<style scoped>
.system-message {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.4rem;
  font-size: 0.75rem;
  color: var(--text-muted);
}

.system-dot {
  width: 4px;
  height: 4px;
  background: currentColor;
  border-radius: 50%;
  flex-shrink: 0;
}
</style>
