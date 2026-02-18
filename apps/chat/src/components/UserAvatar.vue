<script setup>
import { computed } from 'vue'
import { getInitials, getAvatarColor } from '../lib/utils.js'

const props = defineProps({
  name: { type: String, default: '' },
  color: { type: String, default: null },
  size: { type: Number, default: 36 },
  status: { type: String, default: null },
  showStatus: { type: Boolean, default: false },
})

const initials = computed(() => getInitials(props.name))
const bgColor = computed(() => props.color || getAvatarColor(props.name))
const fontSize = computed(() => `${props.size * 0.38}px`)
const dotSize = computed(() => `${Math.max(8, props.size * 0.28)}px`)

const statusColor = computed(() => {
  switch (props.status) {
    case 'online': return 'var(--success, #2a9d8f)'
    case 'away': return '#f77f00'
    case 'dnd': return 'var(--danger, #e63946)'
    case 'invisible': return '#6b7280'
    default: return null
  }
})
</script>

<template>
  <div
    class="avatar-wrapper"
    :style="{ width: size + 'px', height: size + 'px' }"
  >
    <div
      class="avatar"
      :style="{
        width: size + 'px',
        height: size + 'px',
        background: bgColor,
        fontSize,
      }"
    >
      {{ initials }}
    </div>
    <span
      v-if="showStatus && statusColor"
      class="status-dot"
      :style="{
        width: dotSize,
        height: dotSize,
        background: statusColor,
      }"
    ></span>
  </div>
</template>

<style scoped>
.avatar-wrapper {
  position: relative;
  flex-shrink: 0;
}

.avatar {
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 600;
  user-select: none;
}

.status-dot {
  position: absolute;
  bottom: -1px;
  right: -1px;
  border-radius: 50%;
  border: 2px solid var(--bg-secondary, #1a1a2e);
  box-sizing: content-box;
}
</style>
