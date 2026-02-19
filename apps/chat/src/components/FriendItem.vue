<script setup>
import UserAvatar from './UserAvatar.vue'

const props = defineProps({
  friend: { type: Object, required: true },
  isPending: { type: Boolean, default: false },
})

const emit = defineEmits(['message', 'accept', 'decline', 'remove', 'view-profile'])
</script>

<template>
  <div class="friend-item">
    <div class="friend-info" @click="emit('view-profile', friend)">
      <UserAvatar
        :name="friend.name || friend.fromName"
        :color="friend.avatarColor || friend.fromColor"
        :size="28"
        :status="friend.status"
        :show-status="!isPending"
      />
      <span class="friend-name">{{ friend.name || friend.fromName }}</span>
    </div>

    <div class="friend-actions">
      <template v-if="isPending">
        <button class="action-btn accept" title="Accept" aria-label="Accept request" @click="emit('accept', friend.fromId)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
        </button>
        <button class="action-btn decline" title="Decline" aria-label="Decline request" @click="emit('decline', friend.fromId)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </template>
      <template v-else>
        <button class="action-btn" title="Message" aria-label="Message" @click="emit('message', friend)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>
          </svg>
        </button>
        <button class="action-btn remove" title="Remove friend" aria-label="Remove friend" @click="emit('remove', friend.id)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.friend-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.35rem 0.5rem;
  border-radius: 4px;
  transition: background 0.1s;
}

.friend-item:hover {
  background: var(--bg-tertiary);
}

.friend-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
  cursor: pointer;
}

.friend-name {
  font-size: 0.8rem;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.friend-actions {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.1s;
}

@media (hover: hover) and (pointer: fine) {
  .action-btn {
    width: 30px;
    height: 30px;
  }
}

.action-btn svg {
  width: 16px;
  height: 16px;
}

.action-btn:hover {
  background: var(--bg-active);
  color: var(--text-primary);
}

.action-btn:active {
  transform: scale(0.96);
  opacity: 0.8;
}

.action-btn.accept:hover {
  color: var(--success);
}

.action-btn.decline:hover,
.action-btn.remove:hover {
  color: var(--danger);
}
</style>
