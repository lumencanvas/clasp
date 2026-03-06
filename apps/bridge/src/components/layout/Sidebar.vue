<script setup lang="ts">
import { useRouters } from '../../composables/useRouters'
import { useConnections } from '../../composables/useConnections'
import { useDevices } from '../../composables/useDevices'
import { useMonitor } from '../../composables/useMonitor'
import { useRoutes } from '../../composables/useRoutes'
import { useNotifications } from '../../composables/useNotifications'
import StatusDot from '../shared/StatusDot.vue'
import ProtocolBadge from '../shared/ProtocolBadge.vue'
import RouterCard from '../cards/RouterCard.vue'
import ConnectionCard from '../cards/ConnectionCard.vue'
import DeviceCard from '../cards/DeviceCard.vue'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{
  close: []
  'add-router': []
  'add-connection': []
  'edit-router': [router: any]
  'edit-connection': [connection: any]
}>()

const { routers } = useRouters()
const { connections } = useConnections()
const { devices, scanning, scan } = useDevices()
const { signalRate } = useMonitor()
const { routeCount } = useRoutes()
</script>

<template>
  <aside class="sidebar" :class="{ open }">
    <!-- Routers Section -->
    <div class="sidebar-section">
      <div class="section-header">
        <span class="section-title">ROUTERS</span>
        <span class="section-badge">{{ routers.length }}</span>
      </div>
      <div class="device-list" id="router-list">
        <div v-if="routers.length === 0" class="empty-state-small">
          <span class="empty-state-text">No routers configured</span>
        </div>
        <RouterCard v-for="router in routers" :key="router.id" :router="router" @edit="$emit('edit-router', $event)" />
      </div>
      <div class="sidebar-buttons">
        <button class="btn btn-primary btn-sm" @click="$emit('add-router')">+ ROUTER</button>
      </div>
    </div>

    <!-- Protocol Connections Section -->
    <div class="sidebar-section">
      <div class="section-header">
        <span class="section-title">CONNECTIONS</span>
        <span class="section-badge">{{ connections.length }}</span>
      </div>
      <div class="device-list" id="server-list">
        <div v-if="connections.length === 0" class="empty-state-small">
          <span class="empty-state-text">No protocol connections</span>
        </div>
        <ConnectionCard v-for="conn in connections" :key="conn.id" :connection="conn" @edit="$emit('edit-connection', $event)" />
      </div>
      <div class="sidebar-buttons">
        <button class="btn btn-primary btn-sm" @click="$emit('add-connection')">+ CONNECTION</button>
      </div>
    </div>

    <!-- Discovered Devices Section -->
    <div class="sidebar-section">
      <div class="section-header">
        <span class="section-title">DISCOVERED</span>
        <span class="section-badge">{{ devices.length }}</span>
      </div>
      <div class="device-list" id="device-list">
        <div v-if="devices.length === 0" class="empty-state-small">
          <span class="empty-state-text">No devices found</span>
        </div>
        <DeviceCard v-for="device in devices" :key="device.id" :device="device" />
      </div>
      <button class="btn btn-secondary btn-sm" @click="scan" :disabled="scanning">
        <svg v-if="scanning" class="icon spinning" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 11-6.219-8.56"/></svg>
        {{ scanning ? 'SCANNING' : 'SCAN' }}
      </button>
    </div>

    <!-- Stats Section -->
    <div class="sidebar-section sidebar-stats">
      <div class="stat-row">
        <span class="stat-label">Signal Rate</span>
        <span class="stat-value">{{ signalRate }}/s</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Routes</span>
        <span class="stat-value">{{ routeCount }}</span>
      </div>
    </div>
  </aside>
</template>
