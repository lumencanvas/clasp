<script setup lang="ts">
import { useNotifications } from '../../composables/useNotifications'

const { notifications, dismiss } = useNotifications()

function iconFor(type: string): string {
  switch (type) {
    case 'success': return 'V'
    case 'error': return 'X'
    case 'warning': return '!'
    default: return 'i'
  }
}
</script>

<template>
  <div class="notification-container">
    <TransitionGroup name="toast">
      <div
        v-for="n in notifications"
        :key="n.id"
        class="notification"
        :class="`notification-${n.type}`"
      >
        <span class="notification-icon">{{ iconFor(n.type) }}</span>
        <span class="notification-message">{{ n.message }}</span>
        <button class="notification-close" @click="dismiss(n.id)">&times;</button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-enter-active {
  animation: slide-in 0.3s ease-out;
}
.toast-leave-active {
  animation: slide-out 0.3s ease-out forwards;
}
</style>
