<script setup lang="ts">
import type { Router } from '../../lib/types'
import { useRouters } from '../../composables/useRouters'
import StatusDot from '../shared/StatusDot.vue'
import ProtocolBadge from '../shared/ProtocolBadge.vue'

const props = defineProps<{ router: Router }>()
const emit = defineEmits<{ edit: [router: Router] }>()
const { remove, restart } = useRouters()

function statusTitle(r: Router): string {
  if (r.error) return `Error: ${r.error}`
  if (r.isRemote) return `Remote router at ${r.remoteAddress || r.address}`
  switch (r.status) {
    case 'connected':
    case 'running': return 'Running'
    case 'starting': return 'Starting...'
    case 'reconnecting': return 'Reconnecting...'
    case 'error': return 'Error'
    default: return 'Unknown'
  }
}

const hasError = !props.router.isRemote && (props.router.status === 'error' || !!props.router.error)
</script>

<template>
  <div
    class="device-item"
    :class="{ 'device-item-error': hasError, 'device-item-remote': router.isRemote }"
    :title="statusTitle(router)"
  >
    <div class="device-item-main">
      <StatusDot :status="router.isRemote ? 'available' : router.status" />
      <ProtocolBadge :protocol="'clasp'" :remote="router.isRemote" />
      <span class="device-name">{{ router.name }}</span>
    </div>
    <div v-if="router.isRemote" class="device-connection-info">
      {{ router.remoteAddress || router.address }}
    </div>
    <div v-if="hasError" class="device-error-msg">
      {{ router.error || 'Connection error' }}
    </div>
    <div class="device-actions">
      <button v-if="hasError && !router.isRemote" class="btn-device-restart" @click.stop="restart(router.id)" title="Restart router">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 11-2.52-6.24"/><path d="M21 3v6h-6"/></svg>
      </button>
      <button v-if="!router.isRemote" class="btn-device-edit" @click.stop="emit('edit', router)" title="Edit router">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
      </button>
      <button class="btn-device-delete" @click.stop="remove(router.id)" :title="router.isRemote ? 'Remove remote router' : 'Stop router'">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>
  </div>
</template>
