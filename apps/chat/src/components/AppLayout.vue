<script setup>
import { ref } from 'vue'

const props = defineProps({
  showMembers: { type: Boolean, default: false },
})

const emit = defineEmits(['toggle-members'])

const sidebarOpen = ref(false)

function toggleSidebar() {
  sidebarOpen.value = !sidebarOpen.value
}

function closeSidebar() {
  sidebarOpen.value = false
}

defineExpose({ toggleSidebar, closeSidebar })
</script>

<template>
  <div class="app-layout">
    <!-- Mobile sidebar overlay -->
    <div
      v-if="sidebarOpen"
      class="sidebar-overlay"
      @click="closeSidebar"
    ></div>

    <!-- Sidebar -->
    <div :class="['sidebar-column', { open: sidebarOpen }]">
      <slot name="sidebar" :close-sidebar="closeSidebar"></slot>
    </div>

    <!-- Main content -->
    <div class="main-column">
      <slot name="header" :toggle-sidebar="toggleSidebar"></slot>
      <div class="main-content">
        <slot></slot>
      </div>
    </div>

    <!-- Mobile members overlay -->
    <div
      v-if="showMembers"
      class="members-overlay"
      @click="emit('toggle-members')"
    ></div>

    <!-- Members panel -->
    <div v-if="showMembers" :class="['members-column', { open: showMembers }]">
      <div class="members-header">
        <span class="members-title">Members</span>
        <button class="members-close" aria-label="Close members" @click="emit('toggle-members')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
      <slot name="members"></slot>
    </div>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100dvh;
  overflow: hidden;
}

.sidebar-overlay {
  display: none;
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  z-index: 99;
}

.sidebar-column {
  width: 240px;
  flex-shrink: 0;
  display: none;
  z-index: 100;
}

.main-column {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

.main-content {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.members-overlay {
  display: none;
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  z-index: 99;
}

.members-column {
  width: 200px;
  flex-shrink: 0;
  display: none;
  border-left: 1px solid var(--border);
  background: var(--bg-secondary);
}

.members-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 0.75rem;
  height: 52px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.members-title {
  font-size: 0.85rem;
  font-weight: 700;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.members-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  background: none;
  border: none;
  color: var(--text-muted);
  border-radius: 4px;
  cursor: pointer;
}

.members-close svg {
  width: 16px;
  height: 16px;
}

.members-close:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

/* Mobile: sidebar as overlay */
@media (max-width: 767px) {
  .sidebar-column.open {
    display: block;
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: min(280px, 85vw);
    padding-left: env(safe-area-inset-left);
    box-shadow: 4px 0 20px rgba(0,0,0,0.3);
  }

  .sidebar-overlay {
    display: block;
  }
}

/* Tablet+ : sidebar always visible */
@media (min-width: 768px) {
  .sidebar-column {
    display: block;
  }

  .sidebar-overlay {
    display: none !important;
  }
}

/* Mobile/Tablet: members panel as slide-in overlay */
@media (max-width: 1023px) {
  .members-column.open {
    display: flex;
    flex-direction: column;
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: min(280px, 85vw);
    z-index: 100;
    box-shadow: -4px 0 20px rgba(0,0,0,0.3);
  }

  .members-overlay {
    display: block;
  }
}

/* Desktop: members panel inline */
@media (min-width: 1024px) {
  .members-column {
    display: flex;
    flex-direction: column;
  }

  .members-overlay {
    display: none !important;
  }
}
</style>
