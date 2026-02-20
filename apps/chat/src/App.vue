<script setup>
import { ref } from 'vue'
import { useClasp } from './composables/useClasp.js'
import { useNotifications } from './composables/useNotifications.js'
import SplashScreen from './components/SplashScreen.vue'

const { connected, error: claspError } = useClasp()
const { toasts, removeToast } = useNotifications()

const showSplash = ref(!sessionStorage.getItem('clasp-splash-shown'))

function onSplashDone() {
  sessionStorage.setItem('clasp-splash-shown', '1')
  showSplash.value = false
}
</script>

<template>
  <SplashScreen v-if="showSplash" @done="onSplashDone" />
  <router-view />

  <!-- Toast notifications -->
  <div class="toast-container">
    <transition-group name="fade">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        :class="['toast', toast.type, { actionable: toast.action }]"
        @click="() => { try { toast.action?.() } finally { removeToast(toast.id) } }"
      >
        <span class="toast-message">{{ toast.message }}</span>
        <span v-if="toast.action" class="toast-action">View</span>
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
</style>
