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

    <!-- Members panel -->
    <div v-if="showMembers" class="members-column">
      <slot name="members"></slot>
    </div>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100%;
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

.members-column {
  width: 200px;
  flex-shrink: 0;
  display: none;
  border-left: 1px solid var(--border);
  background: var(--bg-secondary);
}

/* Mobile: sidebar as overlay */
@media (max-width: 767px) {
  .sidebar-column.open {
    display: block;
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: 280px;
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

/* Desktop: members panel visible */
@media (min-width: 1024px) {
  .members-column {
    display: flex;
    flex-direction: column;
  }
}
</style>
