<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import ConnectionPanel from '../components/playground/ConnectionPanel.vue'
import ExplorerTab from '../components/playground/ExplorerTab.vue'
import ChatTab from '../components/playground/ChatTab.vue'
import VideoTab from '../components/playground/VideoTab.vue'
import SensorsTab from '../components/playground/SensorsTab.vue'
import SecurityTab from '../components/playground/SecurityTab.vue'
import DiscoveryTab from '../components/playground/DiscoveryTab.vue'
import ConsolePanel from '../components/playground/ConsolePanel.vue'
import { useClasp } from '../composables/useClasp'

const { connected } = useClasp()

const activeTab = ref('explorer')
const consoleOpen = ref(false)
const sidebarOpen = ref(false)
const isDesktop = ref(false)

const tabs = [
  { id: 'explorer', label: 'Explorer', icon: 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5' },
  { id: 'chat', label: 'Chat', icon: 'M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z' },
  { id: 'video', label: 'Video', icon: 'M23 7l-7 5 7 5V7zM1 5h15v14H1z' },
  { id: 'sensors', label: 'Sensors', icon: 'M22 12h-4l-3 9L9 3l-3 9H2' },
  { id: 'security', label: 'Security', icon: 'M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z' },
  { id: 'discovery', label: 'Discovery', icon: 'M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16zM21 21l-4.35-4.35' },
]

function checkDesktop() {
  const wasDesktop = isDesktop.value
  isDesktop.value = window.innerWidth >= 1024
  if (isDesktop.value && !wasDesktop) {
    sidebarOpen.value = false
    consoleOpen.value = true
  }
}

onMounted(() => {
  checkDesktop()
  if (isDesktop.value) {
    consoleOpen.value = true
  }
  window.addEventListener('resize', checkDesktop)
})

onUnmounted(() => {
  window.removeEventListener('resize', checkDesktop)
})

function closeSidebar() {
  sidebarOpen.value = false
}
</script>

<template>
  <div class="playground">
    <div class="playground-header">
      <div class="header-row">
        <button :class="['sidebar-toggle', { connected }]" @click="sidebarOpen = !sidebarOpen">
          <span :class="['toggle-dot', { connected }]"></span>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M6 3v4" /><path d="M10 3v4" /><path d="M4 7h8a1 1 0 0 1 1 1v1a4 4 0 0 1-4 4h0a4 4 0 0 1-4-4V8a1 1 0 0 1 1-1z" /><path d="M8 13v2a4 4 0 0 0 4 4h1a2 2 0 0 0 2-2v0a2 2 0 0 1 2-2h3" />
          </svg>
          <span class="toggle-label">{{ connected ? 'Connected' : 'Connect' }}</span>
        </button>
        <h1>CLASP Playground</h1>
      </div>
      <p class="subtitle">Interactive protocol explorer and testing environment</p>
    </div>

    <!-- Mobile backdrop -->
    <div
      v-if="sidebarOpen && !isDesktop"
      class="sidebar-backdrop"
      @click="closeSidebar"
    ></div>

    <div class="playground-layout">
      <aside :class="['sidebar', { open: sidebarOpen }]">
        <ConnectionPanel />
      </aside>

      <main class="main-content">
        <div class="tab-bar">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            :class="['tab-btn', { active: activeTab === tab.id }]"
            @click="activeTab = tab.id"
          >
            <svg class="tab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path :d="tab.icon" />
            </svg>
            <span class="tab-label">{{ tab.label }}</span>
          </button>
        </div>

        <div class="tab-content">
          <ExplorerTab v-if="activeTab === 'explorer'" />
          <ChatTab v-if="activeTab === 'chat'" />
          <VideoTab v-if="activeTab === 'video'" />
          <SensorsTab v-if="activeTab === 'sensors'" />
          <SecurityTab v-if="activeTab === 'security'" />
          <DiscoveryTab v-if="activeTab === 'discovery'" />
        </div>
      </main>
    </div>

    <div :class="['console-container', { open: consoleOpen }]">
      <button class="console-toggle" @click="consoleOpen = !consoleOpen">
        {{ consoleOpen ? 'Hide' : 'Show' }} Console
      </button>
      <ConsolePanel v-if="consoleOpen" />
    </div>
  </div>
</template>

<style scoped>
.playground {
  min-height: calc(100vh - 60px);
  display: flex;
  flex-direction: column;
}

/* Header */
.playground-header {
  padding: 1rem 1rem 0.75rem;
  border-bottom: 1px solid rgba(0,0,0,0.12);
}

.header-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.playground-header h1 {
  font-size: 1.2rem;
  letter-spacing: 0.2em;
  margin: 0;
}

.playground-header .subtitle {
  display: none;
}

.sidebar-toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-height: 44px;
  padding: 0.4rem 0.75rem;
  background: none;
  border: 1px solid rgba(0,0,0,0.15);
  cursor: pointer;
  flex-shrink: 0;
  transition: border-color 0.2s, background 0.2s;
}

.sidebar-toggle:hover {
  background: rgba(0,0,0,0.03);
}

.sidebar-toggle.connected {
  border-color: rgba(76, 175, 80, 0.4);
  background: rgba(76, 175, 80, 0.06);
}

.sidebar-toggle svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.toggle-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: rgba(0,0,0,0.2);
  transition: background 0.2s;
  flex-shrink: 0;
}

.toggle-dot.connected {
  background: #4CAF50;
  box-shadow: 0 0 6px rgba(76, 175, 80, 0.5);
}

.toggle-label {
  font-size: 0.7rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  font-family: inherit;
  white-space: nowrap;
}

/* Backdrop */
.sidebar-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.3);
  z-index: 40;
}

/* Layout */
.playground-layout {
  display: flex;
  flex: 1;
  min-height: 0;
  position: relative;
}

/* Sidebar: fixed drawer on mobile */
.sidebar {
  position: fixed;
  top: 0;
  left: 0;
  bottom: 0;
  width: 280px;
  z-index: 50;
  border-right: 1px solid rgba(0,0,0,0.12);
  padding: 1.5rem;
  background: var(--paper, #fefcf6);
  transform: translateX(-100%);
  transition: transform 0.25s ease;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
}

.sidebar.open {
  transform: translateX(0);
}

/* Main content */
.main-content {
  display: flex;
  flex-direction: column;
  min-height: 0;
  flex: 1;
  min-width: 0;
}

/* Tab bar */
.tab-bar {
  display: flex;
  gap: 0;
  border-bottom: 1px solid rgba(0,0,0,0.12);
  padding: 0 0.5rem;
  background: rgba(255,255,255,0.2);
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
}

.tab-bar::-webkit-scrollbar {
  display: none;
}

.tab-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.2rem;
  background: none;
  border: none;
  padding: 0.6rem 0.5rem;
  min-height: 44px;
  min-width: 44px;
  font-family: inherit;
  cursor: pointer;
  opacity: 0.5;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: opacity 0.15s, border-color 0.15s;
  flex-shrink: 0;
}

.tab-icon {
  width: 18px;
  height: 18px;
}

.tab-label {
  font-size: 0.6rem;
  letter-spacing: 0.08em;
  white-space: nowrap;
}

.tab-btn:hover {
  opacity: 0.8;
}

.tab-btn.active {
  opacity: 1;
  border-bottom-color: var(--accent);
}

/* Tab content */
.tab-content {
  flex: 1;
  overflow: auto;
  padding: 1rem;
}

/* Console */
.console-container {
  border-top: 1px solid rgba(0,0,0,0.12);
  background: rgba(255,255,255,0.4);
}

.console-container.open {
  height: 150px;
}

.console-toggle {
  display: block;
  width: 100%;
  padding: 0.5rem;
  min-height: 44px;
  background: rgba(0,0,0,0.03);
  border: none;
  font-family: inherit;
  font-size: 0.75rem;
  letter-spacing: 0.15em;
  cursor: pointer;
  opacity: 0.6;
}

.console-toggle:hover {
  opacity: 1;
  background: rgba(0,0,0,0.05);
}

/* 768px+ */
@media (min-width: 768px) {
  .playground-header {
    padding: 1.5rem 4vw 1rem;
  }

  .playground-header h1 {
    font-size: 1.5rem;
  }

  .playground-header .subtitle {
    display: block;
    margin: 0.3rem 0 0;
    opacity: 0.6;
    letter-spacing: 0.05em;
    font-size: 0.85rem;
  }

  .tab-bar {
    padding: 0 1rem;
  }

  .tab-btn {
    flex-direction: row;
    gap: 0.5rem;
    padding: 0.8rem 1rem;
  }

  .tab-label {
    font-size: 0.8rem;
    letter-spacing: 0.12em;
  }

  .tab-content {
    padding: 1.5rem;
  }

  .console-container.open {
    height: 200px;
  }
}

/* 1024px+: desktop layout */
@media (min-width: 1024px) {
  .playground-header {
    padding: 2rem 6vw 1.5rem;
  }

  .playground-header h1 {
    font-size: 1.8rem;
    margin: 0 0 0.5rem;
  }

  .sidebar-toggle {
    display: none;
  }

  .sidebar-backdrop {
    display: none;
  }

  .playground-layout {
    display: grid;
    grid-template-columns: 280px 1fr;
  }

  .sidebar {
    position: static;
    transform: none;
    z-index: auto;
    background: rgba(255,255,255,0.3);
    overflow-y: auto;
  }

  .tab-btn {
    padding: 1rem 1.5rem;
  }

  .tab-label {
    font-size: 0.85rem;
    letter-spacing: 0.15em;
  }

  .tab-icon {
    width: 16px;
    height: 16px;
  }

  .console-container.open {
    height: 250px;
  }
}
</style>
