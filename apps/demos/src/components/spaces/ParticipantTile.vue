<script setup>
import { computed } from 'vue'

const props = defineProps({
  participant: Object,
  size: { type: Number, default: 50 },
  speaking: Object, // { speaking, volume }
  isHost: Boolean, // am I the host?
  handRaised: Boolean,
  mode: { type: String, default: 'speaker' }, // 'speaker' | 'listener'
})
const emit = defineEmits(['promote', 'demote'])

const color = computed(() => props.participant.color || '#888')
const initials = computed(() => {
  const id = props.participant.id || ''
  return id.slice(2, 4).toUpperCase()
})
const isSpeaking = computed(() => props.speaking?.speaking)
const volume = computed(() => props.speaking?.volume || 0)
const ringSize = computed(() => {
  if (!isSpeaking.value) return 0
  return props.size + 8 + volume.value * 14
})
</script>

<template>
  <div class="tile" :class="mode">
    <div class="avatar-wrap" :style="{ width: size + 'px', height: size + 'px' }">
      <div
        v-if="isSpeaking"
        class="avatar-ring"
        :style="{
          width: ringSize + 'px', height: ringSize + 'px',
          borderColor: color, opacity: 0.4 + volume,
          boxShadow: `0 0 ${10 + volume * 20}px ${color}44`,
        }"
      ></div>
      <div
        class="avatar"
        :style="{
          width: size + 'px', height: size + 'px',
          fontSize: (size * 0.35) + 'px',
          background: `linear-gradient(135deg, ${color}33, ${color}11)`,
          border: `2px solid ${color}88`,
          color: color,
        }"
      >{{ initials }}</div>
    </div>
    <div class="name">{{ participant.name }}</div>
    <div v-if="participant.isHost" class="host-tag">host</div>
    <div v-if="isSpeaking && mode === 'speaker'" class="speaking-bars">
      <span class="bar"></span><span class="bar"></span><span class="bar"></span>
    </div>
    <span v-if="handRaised" class="hand-icon">&#x1F44B;</span>
    <button v-if="isHost && mode === 'listener' && handRaised" class="promote-btn" @click.stop="emit('promote', participant.id)">&#8593; speak</button>
    <button v-if="isHost && mode === 'speaker' && !participant.isHost" class="host-action" @click.stop="emit('demote', participant.id)">&#8595; move down</button>
  </div>
</template>

<style scoped>
.tile { display: flex; flex-direction: column; align-items: center; gap: 4px; position: relative; }
.tile.speaker { width: 80px; }
.tile.listener { width: 68px; }
.avatar-wrap { position: relative; flex-shrink: 0; }
.avatar-ring {
  position: absolute; top: 50%; left: 50%;
  transform: translate(-50%, -50%);
  border: 2px solid; border-radius: 50%;
  transition: all 0.1s;
  pointer-events: none;
}
.avatar {
  border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-weight: 700; font-family: 'JetBrains Mono', monospace;
  position: relative; z-index: 1;
}
.name {
  font-size: 10px; color: var(--text2);
  text-align: center; overflow: hidden;
  text-overflow: ellipsis; white-space: nowrap;
  max-width: 100%;
}
.host-tag { font-size: 8px; color: var(--purple); font-family: var(--mono); }
.speaking-bars {
  display: flex; gap: 2px; height: 10px; align-items: flex-end;
}
.bar {
  width: 3px; border-radius: 1px; background: var(--accent);
  animation: barBounce 0.5s ease infinite alternate;
}
.bar:nth-child(1) { height: 4px; }
.bar:nth-child(2) { height: 10px; animation-delay: 0.15s; }
.bar:nth-child(3) { height: 6px; animation-delay: 0.3s; }
@keyframes barBounce { from { transform: scaleY(0.4); } to { transform: scaleY(1); } }
.hand-icon {
  position: absolute; top: -4px; right: 2px;
  font-size: 12px; animation: wave 1s ease infinite alternate;
}
@keyframes wave { from { transform: rotate(-12deg); } to { transform: rotate(12deg); } }
.promote-btn {
  padding: 3px 8px; border-radius: 6px;
  background: var(--accent-dim); color: var(--accent);
  border: 1px solid rgba(0,229,200,0.3);
  font-family: var(--mono); font-size: 9px; font-weight: 600;
  cursor: pointer; transition: all 0.15s;
}
.promote-btn:hover { background: var(--accent); color: var(--bg); }
.host-action {
  padding: 3px 8px; border-radius: 6px;
  background: transparent; color: var(--danger);
  border: 1px solid rgba(255,107,107,0.3);
  font-family: var(--mono); font-size: 9px; font-weight: 600;
  cursor: pointer; transition: all 0.15s;
}
.host-action:hover { background: var(--danger); color: var(--bg); }
</style>
