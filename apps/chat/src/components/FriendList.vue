<script setup>
import FriendItem from './FriendItem.vue'

const props = defineProps({
  friends: { type: Array, default: () => [] },
  pendingRequests: { type: Array, default: () => [] },
})

const emit = defineEmits(['message', 'accept', 'decline', 'remove', 'view-profile'])
</script>

<template>
  <div class="friend-list">
    <div class="friend-entries">
      <!-- Pending requests -->
      <template v-if="pendingRequests.length">
        <div class="section-label">
          Pending Requests
          <span class="count-badge">{{ pendingRequests.length }}</span>
        </div>
        <FriendItem
          v-for="req in pendingRequests"
          :key="req.fromId"
          :friend="req"
          :is-pending="true"
          @accept="emit('accept', $event)"
          @decline="emit('decline', $event)"
        />
      </template>

      <!-- Friends -->
      <div class="section-label">
        All Friends — {{ friends.length }}
      </div>
      <div v-if="!friends.length" class="empty">
        <p>No friends yet</p>
        <span>Click on a member to send a friend request</span>
      </div>
      <FriendItem
        v-for="friend in friends"
        :key="friend.id"
        :friend="friend"
        @message="emit('message', $event)"
        @remove="emit('remove', $event)"
        @view-profile="emit('view-profile', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.friend-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-secondary);
}

.friend-entries {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.section-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.5rem 0.25rem;
  font-size: 0.65rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted);
  font-weight: 700;
}

.count-badge {
  background: var(--accent);
  color: white;
  font-size: 0.6rem;
  padding: 0.05rem 0.35rem;
  border-radius: 8px;
  min-width: 16px;
  text-align: center;
}

.empty {
  padding: 1rem;
  text-align: center;
}

.empty p {
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin-bottom: 0.25rem;
}

.empty span {
  font-size: 0.7rem;
  color: var(--text-muted);
}
</style>
