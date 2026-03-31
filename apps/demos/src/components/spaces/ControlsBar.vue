<script setup>
import { computed } from 'vue'

const props = defineProps({
  role: String, // 'host' | 'speaker' | 'listener'
  isMuted: Boolean,
  chatOpen: Boolean,
  handUp: Boolean,
  myVolume: Number,
})
const emit = defineEmits(['toggle-chat', 'toggle-hand', 'toggle-mic', 'end-room', 'leave'])

const canSpeak = computed(() => props.role === 'host' || props.role === 'speaker')
</script>

<template>
  <div class="controls">
    <button class="ctrl" :class="{ active: chatOpen }" @click="emit('toggle-chat')" title="Chat">
      <svg width="18" height="18" viewBox="0 0 22 22" fill="none"><path d="M4 4h14a2 2 0 012 2v8a2 2 0 01-2 2H8l-4 4V6a2 2 0 012-2z" stroke="currentColor" stroke-width="1.5"/></svg>
    </button>

    <button v-if="role === 'listener'" class="ctrl" :class="{ active: handUp }" @click="emit('toggle-hand')" title="Raise hand">
      <svg width="18" height="18" viewBox="0 0 22 22" fill="none"><path d="M11 2v8M8 5v5M14 5v5M5 9v4a6 6 0 0012 0V9M17 9v4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
    </button>

    <button
      class="mic-btn"
      :class="{ on: !isMuted, disabled: !canSpeak }"
      :disabled="!canSpeak"
      @click="emit('toggle-mic')"
      :style="!isMuted && myVolume > 0 ? { boxShadow: `0 0 ${20 + myVolume * 60}px rgba(0,229,200,${0.3 + myVolume * 0.7})` } : {}"
    >
      <svg v-if="isMuted" width="24" height="24" viewBox="0 0 24 24" fill="none">
        <path d="M1 1l22 22" stroke="#ff6b6b" stroke-width="2" stroke-linecap="round"/>
        <path d="M9 9v3a3 3 0 005.12 2.12M15 9.34V4a3 3 0 00-5.94-.6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M17 16.95A7 7 0 015 12m14 0a7 7 0 01-.11 1.23M12 19v4m-4 0h8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      <svg v-else width="24" height="24" viewBox="0 0 24 24" fill="none">
        <rect x="9" y="2" width="6" height="11" rx="3" stroke="currentColor" stroke-width="2"/>
        <path d="M19 10v2a7 7 0 01-14 0v-2" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        <line x1="12" y1="19" x2="12" y2="23" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        <line x1="8" y1="23" x2="16" y2="23" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
    </button>

    <button v-if="role === 'host'" class="ctrl end" @click="emit('end-room')" title="End room">
      <svg width="18" height="18" viewBox="0 0 20 20" fill="none"><rect x="4" y="4" width="12" height="12" rx="2" stroke="currentColor" stroke-width="1.5"/><line x1="8" y1="8" x2="12" y2="12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><line x1="12" y1="8" x2="8" y2="12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
    </button>

    <button class="ctrl leave" @click="emit('leave')" title="Leave">
      <svg width="18" height="18" viewBox="0 0 22 22" fill="none"><path d="M9 4H5a2 2 0 00-2 2v10a2 2 0 002 2h4M15 7l4 4-4 4M8 11h11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
    </button>
  </div>
</template>

<style scoped>
.controls { display: flex; align-items: center; justify-content: center; gap: 12px; padding: 12px 16px; border-top: 1px solid var(--border); }
.ctrl {
  width: 42px; height: 42px; border-radius: 50%; border: 1px solid var(--border);
  background: var(--surface); color: var(--text2);
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; transition: all 0.15s;
}
.ctrl:hover { border-color: var(--text3); }
.ctrl.active { border-color: var(--accent); color: var(--accent); background: var(--accent-dim); }
.ctrl.end { color: var(--danger); border-color: rgba(255,107,107,0.3); }
.ctrl.end:hover { background: var(--danger); color: var(--bg); }
.ctrl.leave { color: var(--danger); }
.mic-btn {
  width: 56px; height: 56px; border-radius: 50%;
  background: var(--surface2); color: var(--text2); border: none;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; transition: all 0.15s;
}
.mic-btn.on { background: var(--accent); color: var(--bg); }
.mic-btn.disabled { opacity: 0.3; cursor: not-allowed; }
</style>
