<script setup lang="ts">

const props = defineProps<{
  server: {
    id: string
    name: string
    status?: string
    kind: 'router' | 'connection'
    error?: string
    stats?: { messagesIn?: number; messagesOut?: number; uptime?: number }
  }
}>()

function healthStatus(status?: string): string {
  switch (status) {
    case 'connected':
    case 'running': return 'healthy'
    case 'error': return 'unhealthy'
    case 'starting':
    case 'reconnecting': return 'warning'
    default: return 'unhealthy'
  }
}

function healthIcon(status?: string): string {
  switch (healthStatus(status)) {
    case 'healthy': return 'V'
    case 'warning': return '!'
    default: return 'X'
  }
}

function formatUptime(ms?: number): string {
  if (!ms) return '—'
  const s = Math.floor(ms / 1000)
  if (s < 60) return `${s}s`
  if (s < 3600) return `${Math.floor(s / 60)}m`
  return `${Math.floor(s / 3600)}h ${Math.floor((s % 3600) / 60)}m`
}
</script>

<template>
  <div class="server-health-card">
    <div class="server-health-status" :class="healthStatus(server.status)">
      {{ healthIcon(server.status) }}
    </div>
    <div class="server-health-info">
      <div class="server-health-name">{{ server.name }}</div>
      <div class="server-health-stats">
        <span class="server-health-stat">
          Type: <strong>{{ server.kind }}</strong>
        </span>
        <span v-if="server.stats?.messagesIn != null" class="server-health-stat">
          In: <strong>{{ server.stats.messagesIn }}</strong>
        </span>
        <span v-if="server.stats?.messagesOut != null" class="server-health-stat">
          Out: <strong>{{ server.stats.messagesOut }}</strong>
        </span>
        <span v-if="server.stats?.uptime" class="server-health-stat">
          Up: <strong>{{ formatUptime(server.stats.uptime) }}</strong>
        </span>
      </div>
      <div v-if="server.error" style="font-size: 10px; color: var(--color-error);">
        {{ server.error }}
      </div>
    </div>
  </div>
</template>
