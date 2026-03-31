<script setup>
defineProps({ room: Object })
const emit = defineEmits(['join'])
</script>

<template>
  <div class="room-card" @click="emit('join', room.id)">
    <div class="room-top">
      <span class="room-icon">{{ room.icon || '\u25C9' }}</span>
      <span class="room-name">{{ room.name }}</span>
      <span class="room-meta">{{ room.count || 0 }} in room</span>
    </div>
    <div v-if="room.desc" class="room-desc">{{ room.desc }}</div>
    <div class="room-host">hosted by {{ room.hostName || 'unknown' }}</div>
  </div>
</template>

<style scoped>
.room-card {
  padding: 14px 16px; background: var(--surface); border: 1px solid var(--border);
  border-radius: 12px; cursor: pointer; display: flex; flex-direction: column; gap: 6px;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.room-card:hover { border-color: var(--accent); box-shadow: 0 0 20px rgba(0,229,200,0.06); }
.room-top { display: flex; align-items: center; gap: 8px; }
.room-icon { font-size: 14px; color: var(--accent); }
.room-name { font-size: 15px; font-weight: 700; color: var(--text); flex: 1; }
.room-meta { font-family: var(--mono); font-size: 11px; color: var(--text3); }
.room-desc { font-size: 13px; color: var(--text2); padding-left: 32px; }
.room-host { font-family: var(--mono); font-size: 11px; color: var(--text3); padding-left: 32px; }
</style>
