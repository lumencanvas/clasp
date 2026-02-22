<script setup>
import { ref } from 'vue'

const props = defineProps({
  title: { type: String, required: true },
  defaultOpen: { type: Boolean, default: false },
  badge: { type: String, default: '' }
})

const open = ref(props.defaultOpen)
</script>

<template>
  <div class="config-section" :class="{ open }">
    <div class="config-section-header" @click="open = !open">
      <span class="config-section-chevron" />
      <span class="config-section-title">{{ title }}</span>
      <span v-if="badge" class="config-section-badge">{{ badge }}</span>
    </div>
    <div class="config-section-body">
      <div class="config-section-inner">
        <slot />
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-section {
  border: 1px solid var(--border);
  margin-bottom: 0.5rem;
}

.config-section-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.65rem 1rem;
  cursor: pointer;
  user-select: none;
  transition: background 0.15s;
}

.config-section-header:hover {
  background: var(--hover-bg);
}

.config-section-chevron {
  display: inline-block;
  font-size: 0.7rem;
  transition: transform 0.2s ease;
  opacity: 0.5;
  flex-shrink: 0;
}

.config-section-chevron::before {
  content: '\25B8';
}

.config-section.open .config-section-chevron {
  transform: rotate(90deg);
}

.config-section-title {
  font-family: 'Archivo Black', sans-serif;
  font-size: 0.72rem;
  letter-spacing: 0.2em;
  text-transform: uppercase;
}

.config-section-badge {
  margin-left: auto;
  font-size: 0.65rem;
  padding: 0.1rem 0.4rem;
  background: var(--accent);
  color: #fff;
  border-radius: 2px;
  letter-spacing: 0.06em;
}

.config-section-body {
  max-height: 0;
  overflow: hidden;
  transition: max-height 0.3s ease;
}

.config-section.open .config-section-body {
  max-height: 5000px;
}

.config-section-inner {
  padding: 0.5rem 1rem 1rem;
  border-top: 1px solid var(--border);
}
</style>
