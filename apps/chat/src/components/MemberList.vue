<script setup>
import MemberItem from './MemberItem.vue'

const props = defineProps({
  members: { type: Array, default: () => [] },
  roomId: { type: String, default: null },
})

const emit = defineEmits(['view-profile'])
</script>

<template>
  <div class="member-list">
    <div class="member-header">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
        <circle cx="9" cy="7" r="4"/>
        <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
        <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
      </svg>
      Members — {{ members.length }}
    </div>
    <div class="member-entries">
      <MemberItem
        v-for="member in members"
        :key="member.id"
        :member="member"
        :room-id="roomId"
        @view-profile="emit('view-profile', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.member-list {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.member-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem;
  font-size: 0.65rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--text-muted);
  font-weight: 700;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.member-header svg {
  width: 14px;
  height: 14px;
}

.member-entries {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}
</style>
