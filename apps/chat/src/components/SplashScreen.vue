<script setup>
import { ref, onMounted } from 'vue'

const emit = defineEmits(['done'])
const visible = ref(true)
const fading = ref(false)

onMounted(() => {
  // Animation completes at ~2.05s (last pop at 1.75s + 0.3s duration)
  // Wait a beat, then fade out
  setTimeout(() => {
    fading.value = true
    setTimeout(() => {
      visible.value = false
      emit('done')
    }, 400)
  }, 2200)
})
</script>

<template>
  <div v-if="visible" :class="['splash-overlay', { fading }]">
    <img src="/logo-animated.svg" alt="CLASP Chat" class="splash-logo" />
  </div>
</template>

<style scoped>
.splash-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary, #0a0a0a);
  transition: opacity 0.4s ease-out;
}

.splash-overlay.fading {
  opacity: 0;
}

.splash-logo {
  width: 180px;
  height: 180px;
}

@media (max-width: 480px) {
  .splash-logo {
    width: 140px;
    height: 140px;
  }
}
</style>
