<script setup lang="ts">
import type { DirectLink } from '../../lib/types'
import { useBridges } from '../../composables/useBridges'
import ProtocolBadge from '../shared/ProtocolBadge.vue'

const props = defineProps<{ bridge: DirectLink }>()
const emit = defineEmits<{ edit: [bridge: DirectLink] }>()
const { remove } = useBridges()
</script>

<template>
  <div class="bridge-card" :data-active="bridge.active !== false">
    <div class="bridge-endpoint">
      <div class="bridge-endpoint-label">SOURCE</div>
      <ProtocolBadge :protocol="bridge.source || 'unknown'" />
      <div class="bridge-endpoint-value">{{ bridge.sourceAddr || '—' }}</div>
    </div>
    <svg class="bridge-arrow" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/>
    </svg>
    <div class="bridge-endpoint">
      <div class="bridge-endpoint-label">TARGET</div>
      <ProtocolBadge :protocol="bridge.target || 'unknown'" />
      <div class="bridge-endpoint-value">{{ bridge.targetAddr || '—' }}</div>
    </div>
    <div class="bridge-actions">
      <button class="btn btn-sm btn-secondary" @click="emit('edit', bridge)" title="Edit link">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
      </button>
      <button class="btn btn-sm btn-delete" @click="remove(bridge.id)" title="Delete link">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>
  </div>
</template>
