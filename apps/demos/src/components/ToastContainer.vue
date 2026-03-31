<script setup>
import { useToast } from '../composables/useToast.js'
const { toasts } = useToast()
</script>

<template>
  <div class="toasts">
    <div v-for="t in toasts" :key="t.id" class="toast" :class="[t.type, { out: t.out }]">{{ t.msg }}</div>
  </div>
</template>

<style scoped>
.toasts {
  position: fixed;
  bottom: 20px;
  right: 20px;
  z-index: 600;
  display: flex;
  flex-direction: column;
  gap: 6px;
  pointer-events: none;
}
.toast {
  background: #161616;
  border: 1px solid rgba(255,255,255,0.13);
  border-radius: 5px;
  padding: 9px 13px;
  font-family: 'JetBrains Mono', 'Space Mono', monospace;
  font-size: 11px;
  color: #e8e8e8;
  max-width: 280px;
  min-width: 160px;
  animation: tin 0.25s ease;
  pointer-events: auto;
}
.toast.wrn { border-color: rgba(212,168,64,0.3); color: #d4a840; }
.toast.err { border-color: rgba(212,68,68,0.3); color: #d44444; }
@keyframes tin { from { opacity: 0; transform: translateX(12px); } to { opacity: 1; transform: none; } }
.toast.out { animation: tout 0.25s ease forwards; }
@keyframes tout { to { opacity: 0; transform: translateX(12px); } }
</style>
