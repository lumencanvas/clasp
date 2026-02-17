<script setup>
import { computed } from 'vue'
import { getInitials, getAvatarColor } from '../lib/utils.js'

const props = defineProps({
  name: { type: String, default: '' },
  color: { type: String, default: null },
  size: { type: Number, default: 36 },
})

const initials = computed(() => getInitials(props.name))
const bgColor = computed(() => props.color || getAvatarColor(props.name))
const fontSize = computed(() => `${props.size * 0.38}px`)
</script>

<template>
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
</template>

<style scoped>
.avatar {
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 600;
  flex-shrink: 0;
  user-select: none;
}
</style>
