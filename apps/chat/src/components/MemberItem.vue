<script setup>
import { useClasp } from '../composables/useClasp.js'
import UserAvatar from './UserAvatar.vue'

const props = defineProps({
  member: { type: Object, required: true },
})

const { sessionId } = useClasp()
</script>

<template>
  <div :class="['member-item', { self: member.id === sessionId }]">
    <UserAvatar :name="member.name" :color="member.avatarColor" :size="28" />
    <span class="member-name">{{ member.name }}</span>
    <span v-if="member.id === sessionId" class="you-tag">you</span>
    <span class="online-dot"></span>
  </div>
</template>

<style scoped>
.member-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.5rem;
  border-radius: 4px;
  transition: background 0.1s;
}

.member-item:hover {
  background: var(--bg-tertiary);
}

.member-item.self {
  background: rgba(230,57,70,0.06);
}

.member-name {
  flex: 1;
  font-size: 0.8rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
}

.you-tag {
  font-size: 0.6rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  flex-shrink: 0;
}

.online-dot {
  width: 7px;
  height: 7px;
  background: var(--success);
  border-radius: 50%;
  flex-shrink: 0;
}
</style>
