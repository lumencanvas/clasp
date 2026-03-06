<script setup lang="ts">
import { computed } from 'vue'
import type { Connection } from '../../lib/types'
import { protocolNames } from '../../lib/constants'
import { useConnections } from '../../composables/useConnections'
import { useRouters } from '../../composables/useRouters'
import StatusDot from '../shared/StatusDot.vue'
import ProtocolBadge from '../shared/ProtocolBadge.vue'

const props = defineProps<{ connection: Connection }>()
const emit = defineEmits<{ edit: [connection: Connection] }>()
const { remove, restart } = useConnections()
const { routers } = useRouters()

const hasError = computed(() => props.connection.status === 'error' || !!props.connection.error)

const routerInfo = computed(() => {
  const c = props.connection
  const id = c.connectedRouterId || c.routerId
  if (id) return routers.value.find(r => r.id === id) || null
  return routers.value.find(r => r.status === 'running' || r.status === 'connected') || null
})

function statusTitle(c: Connection): string {
  if (c.error) return `Error: ${c.error}`
  switch (c.status) {
    case 'connected':
    case 'running': return 'Running'
    case 'starting': return 'Starting...'
    case 'reconnecting': return 'Reconnecting...'
    case 'error': return 'Error'
    default: return 'Disconnected'
  }
}
</script>

<template>
  <div class="device-item" :class="{ 'device-item-error': hasError }" :title="statusTitle(connection)">
    <div class="device-item-main">
      <StatusDot :status="connection.status" />
      <ProtocolBadge :protocol="connection.protocol || connection.type" />
      <span class="device-name">{{ connection.name }}</span>
    </div>
    <div v-if="routerInfo" class="device-connection-info">
      -> Router: {{ routerInfo.name }}
      <span v-if="connection.routerConnected" class="router-status-badge connected" title="Connected to router">*</span>
      <span v-else-if="connection.routerError" class="router-status-badge error" title="Router connection failed">!</span>
    </div>
    <div v-if="connection.routerError" class="device-error-msg">
      Router connection: {{ connection.routerError }}
    </div>
    <div v-if="hasError" class="device-error-msg">
      {{ connection.error || 'Connection error' }}
    </div>
    <div class="device-actions">
      <button v-if="hasError" class="btn-device-restart" @click.stop="restart(connection.id)" title="Restart connection">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 11-2.52-6.24"/><path d="M21 3v6h-6"/></svg>
      </button>
      <button class="btn-device-edit" @click.stop="emit('edit', connection)" title="Edit connection">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
      </button>
      <button class="btn-device-delete" @click.stop="remove(connection.id)" title="Stop connection">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>
  </div>
</template>
