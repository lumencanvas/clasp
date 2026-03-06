<script setup lang="ts">
import { ref } from 'vue'
import Sidebar from './Sidebar.vue'
import TabBar from './TabBar.vue'
import RouterModal from '../modals/RouterModal.vue'
import ConnectionModal from '../modals/ConnectionModal.vue'
import ExportModal from '../modals/ExportModal.vue'
import { useMonitor } from '../../composables/useMonitor'
import { useDiagnostics } from '../../composables/useDiagnostics'
import { useRouters } from '../../composables/useRouters'
import { useDevices } from '../../composables/useDevices'
import { useConfig } from '../../composables/useConfig'
import { useNotifications } from '../../composables/useNotifications'

const { signalRate } = useMonitor()
const { bridgeServiceReady } = useDiagnostics()
const { routers } = useRouters()
const { devices } = useDevices()
const { exportToFile, importFromFile } = useConfig()
const { notify } = useNotifications()

const sidebarOpen = ref(false)
const routerModalRef = ref<InstanceType<typeof RouterModal> | null>(null)
const connectionModalRef = ref<InstanceType<typeof ConnectionModal> | null>(null)
const exportModalRef = ref<InstanceType<typeof ExportModal> | null>(null)

function toggleSidebar() {
  sidebarOpen.value = !sidebarOpen.value
}

function closeSidebar() {
  sidebarOpen.value = false
}

function openRouterModal(router?: any) {
  routerModalRef.value?.open(router)
}

function openConnectionModal(connection?: any) {
  connectionModalRef.value?.open(connection)
}

async function handleExport() {
  try {
    await exportToFile()
  } catch (e: any) {
    notify(`Export failed: ${e.message || e}`, 'error')
  }
}

async function handleImport() {
  try {
    await importFromFile()
  } catch (e: any) {
    notify(`Import failed: ${e.message || e}`, 'error')
  }
}
</script>

<template>
  <div id="app">
    <header class="titlebar">
      <div class="titlebar-drag"></div>
      <div class="titlebar-content">
        <button class="titlebar-btn sidebar-toggle" @click="toggleSidebar" title="Toggle sidebar">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="3" y1="6" x2="21" y2="6" /><line x1="3" y1="12" x2="21" y2="12" /><line x1="3" y1="18" x2="21" y2="18" />
          </svg>
        </button>
        <svg class="titlebar-icon" width="16" height="16" viewBox="0 0 32 32">
          <path d="M 4 16 L 4 6 Q 4 2, 8 2 L 13 2 L 13 5 L 9 5 Q 7 5, 7 8 L 7 16 L 7 24 Q 7 27, 9 27 L 13 27 L 13 30 L 8 30 Q 4 30, 4 26 Z" fill="currentColor"/>
          <path d="M 28 16 L 28 6 Q 28 2, 24 2 L 19 2 L 19 5 L 23 5 Q 25 5, 25 8 L 25 16 L 25 24 Q 25 27, 23 27 L 19 27 L 19 30 L 24 30 Q 28 30, 28 26 Z" fill="currentColor"/>
          <line x1="7" y1="11" x2="25" y2="11" stroke="currentColor" stroke-width="2"/>
          <line x1="7" y1="16" x2="25" y2="16" stroke="currentColor" stroke-width="2"/>
          <line x1="7" y1="21" x2="25" y2="21" stroke="currentColor" stroke-width="2"/>
          <circle cx="16" cy="16" r="3" fill="none" stroke="currentColor" stroke-width="2"/>
          <circle cx="16" cy="16" r="1" fill="#FF5F1F"/>
        </svg>
        <h1 class="titlebar-title">CLASP BRIDGE</h1>
      </div>
      <div class="titlebar-actions">
        <button class="titlebar-btn" @click="handleImport" title="Import Config">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        </button>
        <button class="titlebar-btn" @click="handleExport" title="Export JSON Config">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
        </button>
        <button class="titlebar-btn" @click="exportModalRef?.open()" title="Export CLI/Docker/Client">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
        </button>
        <span class="titlebar-version">v4.0.3</span>
      </div>
    </header>

    <div class="app-layout">
      <Sidebar
        :open="sidebarOpen"
        @close="closeSidebar"
        @add-router="openRouterModal"
        @add-connection="openConnectionModal"
        @edit-router="openRouterModal"
        @edit-connection="openConnectionModal"
      />
      <div v-if="sidebarOpen" class="sidebar-backdrop" @click="closeSidebar"></div>

      <div class="main-content">
        <TabBar />
        <div class="tab-panels">
          <router-view />
        </div>
      </div>
    </div>

    <footer class="statusbar">
      <div class="status-group">
        <span class="status-indicator" :class="{ connected: bridgeServiceReady }"></span>
        <span class="status-text">
          <strong>{{ routers.length }}</strong> routers
        </span>
      </div>
      <div class="status-group">
        <span class="status-text">
          <strong>{{ devices.length }}</strong> devices
        </span>
      </div>
      <span class="status-spacer"></span>
      <div class="status-group">
        <span class="status-text"><strong>{{ signalRate }}</strong>/s</span>
      </div>
    </footer>

    <RouterModal ref="routerModalRef" />
    <ConnectionModal ref="connectionModalRef" />
    <ExportModal ref="exportModalRef" />
  </div>
</template>

<style scoped>
.sidebar-backdrop {
  display: none;
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 99;
}

.sidebar-backdrop.visible,
.sidebar-backdrop {
  display: block;
}

.sidebar-toggle {
  display: none;
}

@media (max-width: 600px) {
  .sidebar-toggle {
    display: flex;
  }
}
</style>
