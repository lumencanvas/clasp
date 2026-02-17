<script setup>
import { useClasp } from './composables/useClasp.js'
import { useNotifications } from './composables/useNotifications.js'

const { connected, error: claspError } = useClasp()
const { toasts, removeToast } = useNotifications()
</script>

<template>
  <router-view />

  <!-- Toast notifications -->
  <div class="toast-container">
    <transition-group name="fade">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        :class="['toast', toast.type]"
        @click="removeToast(toast.id)"
      >
        {{ toast.message }}
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
</style>
